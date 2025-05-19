<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import Chart from 'chart.js/auto';
  import { page } from '$app/state';
  import { docs } from '$lib/stores/docs';
  import { get } from 'svelte/store';
  
  // reactivity trigger for $effect(), just whenever this property is reset
  let { docId = '' } = $props();

  let canvasEl: HTMLCanvasElement;
  let chart: Chart | null = null;

  $effect(() => {
    
    //should ensure that docs is populated... but maybe works for now with reactivity? since i have $? whenever docs is reloaded, the chart should too
    
    const revisions = $docs.find(d => d.doc_id === docId)?.revision_summary ?? [];
    
    if (chart) chart.destroy();
  
    const bins = new Map<string, { added: number; deleted: number }>();
  
    for (const rev of revisions) {
      const date = new Date(rev.revision_time);
      const rounded = new Date(Math.floor(date.getTime() / (5 * 60 * 1000)) * (5 * 60 * 1000));
      const key = rounded.toISOString();
  
      if (!bins.has(key)) bins.set(key, { added: 0, deleted: 0 });
      bins.get(key)!.added += rev.added_words;
      bins.get(key)!.deleted += rev.deleted_words ?? 0;
    }
  
    const labels = Array.from(bins.keys()).sort();
    const addedData = labels.map(l => bins.get(l)!.added);
    const deletedData = labels.map(l => -1 * bins.get(l)!.deleted); // negative = downward bars
  
    const ctx = canvasEl.getContext('2d');
    if (!ctx) return;
  
    chart = new Chart(ctx, {
      type: 'bar',
      data: {
        labels,
        datasets: [
          {
          label: 'Added',
          data: addedData,
          backgroundColor: 'rgba(34, 197, 94, 0.6)' // Tailwind green-500
          },
          {
          label: 'Deleted',
          data: deletedData,
          backgroundColor: 'rgba(239, 68, 68, 0.6)' // Tailwind red-500
          }
        ]
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        animation: false,
        plugins: {
          legend: { display: false },
          tooltip: { enabled: true }
        },
        scales: {
          x: {
            ticks: {
              autoSkip: true,
              maxTicksLimit: 4,
              callback: function (value, index, ticks) {
              const label = this.getLabelForValue(value);
              // Format ISO timestamp to hh:mm
              const date = new Date(label);
              return `${date.getHours()}:${String(date.getMinutes()).padStart(2, '0')}`;
              }
            },
            title: {
              display: true,
              text: 'Time',
              color: '#666',
              font: {
                size: 10
              }
            },
            grid: {
              display: false
            }
          },
          y: {
            title: {
              display: true,
              text: 'Words Written/Deleted',
              color: '#666',
              font: { size: 10 }
            },
            ticks: {
              callback: function (value) {
                return Math.abs(value as number); // Hide negative signs
              },
              stepSize: 10,
              font: {
                size: 8
              }
            },
            grid: {
              drawBorder: false
            }
          }
        }
      }
    });
  });

  onDestroy(() => {
    if (chart) {
      chart.destroy();
      chart = null;
    }
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

<div class="plot-wrapper">
  <canvas bind:this={canvasEl}></canvas>
</div>
