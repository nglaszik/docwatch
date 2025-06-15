<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Button, Dropdown, DropdownItem, Radio } from "flowbite-svelte";
  import { ChevronDownOutline } from "flowbite-svelte-icons";
  import 'chartjs-adapter-date-fns';
  import Chart from 'chart.js/auto';
  import { docs } from '$lib/stores/docs';

  // reactivity
  let { revisions = [], docId = '' } = $props();
  let mode = $state('Time Series – Added'); // Time Series – Added, Time Series – Deleted, Histogram – Added, Histogram – Deleted
  let range = $state('All'); // today, week, month, year, all

  let canvasEl: HTMLCanvasElement;
  let chart: Chart | null = null;

  const timeRanges = {
    Today: 1,
    Week: 7,
    Month: 30,
    Year: 365,
    All: Infinity
  };
  
  function exportRevisionCSV(): string {
    
    const rows = [
      'Time,Added Words,Deleted Words'
    ];
  
    for (const rev of revisions) {
      const time = new Date(rev.revision_time).toISOString();
      rows.push(`${time},${rev.added_words},${rev.deleted_words ?? 0}`);
    }
  
    return rows.join('\n');
  }


  function getRangeStart(days: number) {
    const now = new Date();
    return new Date(now.getTime() - days * 24 * 60 * 60 * 1000);
  }

  $effect(() => {
    
    if (!canvasEl) return;

    if (chart) chart.destroy();

    const bins = new Map<string, { added: number; deleted: number }>();
    const addedWords: number[] = [];
    const deletedWords: number[] = [];

    for (const rev of revisions) {
      const date = new Date(rev.revision_time);
      const rounded = new Date(Math.floor(date.getTime() / (5 * 60 * 1000)) * (5 * 60 * 1000));
      const key = rounded.toISOString();

      if (!bins.has(key)) bins.set(key, { added: 0, deleted: 0 });
      bins.get(key)!.added += rev.added_words;
      bins.get(key)!.deleted += rev.deleted_words;

      addedWords.push(rev.added_words);
      deletedWords.push(rev.deleted_words);
    }

    const labels = Array.from(bins.keys()).sort();
    const addedData = labels.map(l => bins.get(l)!.added);
    const deletedData = labels.map(l => bins.get(l)!.deleted);
    
    let rangeStart: Date;
    if (range === 'All') {
      const allTimes = revisions.map(r => new Date(r.revision_time));
      rangeStart = allTimes.length > 0
        ? new Date(Math.min(...allTimes.map(d => d.getTime())))
        : new Date(); // fallback to now if no data
    } else {
      rangeStart = getRangeStart(timeRanges[range]);
    }
    
    let timeUnit = 'minute';
    let stepSize = 5;
    let xMin: string;
    let xMax: string;
    
    if (range === 'Today') {
      timeUnit = 'minute';
      stepSize = 5;
      xMin = rangeStart.toISOString();
      xMax = new Date().toISOString();
    } else if (range === 'Week') {
      timeUnit = 'hour';
      stepSize = 6;
      xMin = rangeStart.toISOString();
      xMax = new Date().toISOString();
    } else if (range === 'Month') {
      timeUnit = 'day';
      stepSize = 2;
      xMin = rangeStart.toISOString();
      xMax = new Date().toISOString();
    } else if (range === 'Year') {
      timeUnit = 'month';
      stepSize = 1;
      xMin = rangeStart.toISOString();
      xMax = new Date().toISOString();
    } else if (range === 'All') {
      const allTimes = revisions.map(r => new Date(r.revision_time));
      const sorted = allTimes.sort((a, b) => a.getTime() - b.getTime());
      const start = sorted[0] ?? new Date();
      const end = sorted[sorted.length - 1] ?? new Date();
      const spanMs = end.getTime() - start.getTime();
      const spanDays = spanMs / (1000 * 60 * 60 * 24);
      xMin = start.toISOString();
      xMax = end.toISOString();
      
      if (spanDays < 1) {
        timeUnit = 'minute';
        stepSize = 5;
      } else if (spanDays < 7) {
        timeUnit = 'hour';
        stepSize = 2;
      } else if (spanDays < 30) {
        timeUnit = 'day';
        stepSize = 1;
      } else if (spanDays < 365) {
        timeUnit = 'week';
        stepSize = 1;
      } else {
        timeUnit = 'month';
        stepSize = 1;
      }
    }
    
    const filteredLabels = labels.filter(l => new Date(l) >= rangeStart);
    const filteredAdded = filteredLabels.map(l => bins.get(l)!.added);
    const filteredDeleted = filteredLabels.map(l => bins.get(l)!.deleted);

    const ctx = canvasEl.getContext('2d');
    if (!ctx) return;

    let chartData = {};
    let title = '';

    if (mode.startsWith('Time')) {
      chartData = {
        datasets: [{
          label: mode === 'Time Series – Added' ? 'Added Words' : 'Deleted Words',
          data: filteredLabels.map((label, i) => ({
            x: label,
            y: mode === 'Time Series – Added' ? bins.get(label)!.added : bins.get(label)!.deleted
          })),
          backgroundColor: mode === 'Time Series – Added'
            ? 'rgba(34, 197, 94, 0.6)'
            : 'rgba(239, 68, 68, 0.6)',
          barPercentage: 1.0,
          categoryPercentage: 1.0
        }]
      };
      title = 'Words over Time';
    } else {
      const histData = mode === 'Histogram – Added' ? addedWords : deletedWords;
      const binSize = 10;
      
      // Create a bucket count
      const buckets: { [bin: number]: number } = {};
      let maxBin = 0;
      for (const w of histData) {
        const bin = Math.floor(w / binSize);
        buckets[bin] = (buckets[bin] || 0) + 1;
        if (bin > maxBin) maxBin = bin;
      }
      
      // Generate x/y points for Chart.js scatter/line style with bars
      const x_vals: number[] = [];
      const y_vals: number[] = [];
      for (let i = 0; i <= maxBin; i++) {
        x_vals.push(i * binSize + binSize / 2); // center of bin
        y_vals.push(buckets[i] || 0); // default to 0 for gaps
      }
      
     const data = x_vals.map((x, i) => ({ x, y: y_vals[i] }));
     
     chartData = {
       datasets: [{
         label: mode === 'Histogram – Added' ? 'Added Words (Histogram)' : 'Deleted Words (Histogram)',
         data,
         backgroundColor: mode === 'Histogram – Added'
           ? 'rgba(34, 197, 94, 0.6)'
           : 'rgba(239, 68, 68, 0.6)',
         barPercentage: 1.0,
         categoryPercentage: 1.0,
       }]
     };
     
      title = 'Words per Revision';
    }

    chart = new Chart(ctx, {
      type: 'bar',
      data: chartData,
      options: {
        responsive: true,
        maintainAspectRatio: false,
        animation: false,
        plugins: {
          legend: { display: true },
          tooltip: { enabled: true },
          title: { display: true, text: title },
          zoom: {
            zoom: {
              wheel: {
                enabled: true,
              },
              pinch: {
                enabled: true
              },
              mode: 'x',
            }
          }
        },
        scales: {
          x: mode.startsWith('Time')
          ? {
              type: 'time',
              time: {
                unit: timeUnit,
                stepSize,
                displayFormats: {
                  minute: 'M/d HH:mm',
                  hour: 'M/d HH:mm',
                  day: 'M/d',
                  month: 'MMM yyyy'
                }
              },
              min: xMin,
              max: xMax,
              ticks: {
                autoSkip: true,
                font: { size: 8 }
              },
              title: {
                display: true,
                text: 'Time',
                color: '#666',
                font: { size: 10 }
              },
              grid: {
                display: false
              }
            }
          : {
              type: 'linear',
              offset: false,
              ticks: {
                stepSize: 10,
                font: { size: 8 }
              },
              title: {
                display: true,
                text: 'Word Count Bin',
                color: '#666',
                font: { size: 10 }
              },
              grid: { display: false, offset: false }
            },
          y: {
            beginAtZero: true,
            ticks: {
              font: { size: 8 }
            }
          }
        }
      }
    });
  });

  onDestroy(() => {
    if (chart) chart.destroy();
  });
</script>

<style>
  .plot-wrapper {
    width: 100%;
    max-width: 100%;
    height: 300px;
    box-sizing: border-box;
  }
  canvas {
    width: 100% !important;
    height: 100% !important;
  }
</style>

<div class="w-full">
  <div class="flex justify-end gap-4 mb-2 pr-4">
    
    <Button onclick={() => {
      const csv = exportRevisionCSV();
      const blob = new Blob([csv], { type: 'text/csv' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `doc-${docId}-revisions.csv`;
      a.click();
      URL.revokeObjectURL(url);
    }}>
      Download CSV
    </Button>
    
    <Button>
      {mode}<ChevronDownOutline class="w-4 ms-2 h-6 text-white dark:text-white" />
    </Button>
    <Dropdown simple class="w-52 space-y-3 p-3 text-sm">
      <li>
        <Radio name="mode" bind:group={mode} value={'Time Series – Added'}>Time Series – Added</Radio>
      </li>
      <li>
        <Radio name="mode" bind:group={mode} value={'Time Series – Deleted'}>Time Series – Deleted</Radio>
      </li>
      <li>
        <Radio name="mode" bind:group={mode} value={'Histogram – Added'}>Histogram – Added</Radio>
      </li>
      <li>
        <Radio name="mode" bind:group={mode} value={'Histogram – Deleted'}>Histogram – Deleted</Radio>
      </li>
    </Dropdown>
  
    <Button>
      {range}<ChevronDownOutline class="ms-2 h-6 w-4 text-white dark:text-white" />
    </Button>
    <Dropdown simple class="w-32 space-y-3 p-3 text-sm">
      <li>
        <Radio name="range" bind:group={range} value={'Today'}>Today</Radio>
      </li>
      <li>
        <Radio name="range" bind:group={range} value={'Week'}>This Week</Radio>
      </li>
      <li>
        <Radio name="range" bind:group={range} value={'Month'}>This Month</Radio>
      </li>
      <li>
        <Radio name="range" bind:group={range} value={'Year'}>This Year</Radio>
      </li>
      <li>
        <Radio name="range" bind:group={range} value={'All'}>All</Radio>
      </li>
    </Dropdown>
    
  </div>
  
  <!-- Chart -->
  <div class="plot-wrapper">
    <canvas bind:this={canvasEl}></canvas>
  </div>
  
</div>
