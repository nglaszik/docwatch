<script lang="ts">
  
  import { fetchRevisions, fetchDiff } from '$lib/api/docs';
  import { Button, Breadcrumb, BreadcrumbItem } from 'flowbite-svelte';
  import RevisionMiniPlot from '$lib/components/RevisionMiniPlot.svelte';
  import { page } from '$app/stores';
  import { docs } from '$lib/stores/docs';
  import { get } from 'svelte/store';
  import { SvelteMap } from 'svelte/reactivity';
  import { base } from '$app/paths';
  
  type DiffBlock = { type: 'add' | 'del' | 'neutral'; text: string };
  
  // revisions reacts whenever doc_id changes, then html awaits response to render
  const revisions = $derived(fetchRevisions($page.params.doc_id));
  
  let expandedDiffs = new SvelteMap();
  
  async function loadAndToggle(revId: number) {
    if (!expandedDiffs.has(revId)) {
      const diff = await fetchDiff(revId);
      expandedDiffs.set(revId, diff);
    } else {
      expandedDiffs.delete(revId);
    }
  }
  
  function formatTime(iso) {
    const d = new Date(iso);
    return d.toLocaleString();
  }
  
</script>

{#await revisions}
{:then revisions}

  <div class="p-6 space-y-6">
    
    <Breadcrumb aria-label="Default breadcrumb example">
      <BreadcrumbItem href={`${base}/u/folders/home`} home>Home</BreadcrumbItem>
      {#each $docs.breadcrumbs as breadcrumb}
        <BreadcrumbItem href={`${base}/u/folders/${breadcrumb.id}`}>{breadcrumb.name}</BreadcrumbItem>
      {/each}
    </Breadcrumb>
    
    <div class="w-full max-w-screen-lg space-y-6">
      <div class="w-full h-80 rounded flex items-center justify-center">
        <RevisionMiniPlot revisions={revisions ?? []} docId={$page.params.doc_id ?? ''} />
      </div>
      <div class="text-black dark:text-white">{revisions.length} Revisions</div>
      <div class="w-full mx-auto p-4 space-y-4">
        {#each revisions as rev}
          <div class="border rounded p-4">
            <div class="flex justify-between items-center text-sm font-medium mb-1 text-black dark:text-white">
              <span>{formatTime(rev.revision_time)} — {rev.added_words} added, {rev.deleted_words} deleted</span>
              <Button onclick={() => loadAndToggle(rev.id)}>
                {expandedDiffs.has(rev.id) ? 'Hide' : 'Show'} Diff
              </Button>
            </div>
        
            {#if expandedDiffs.has(rev.id)}
              <div class="overflow-y-auto border rounded p-2 text-sm leading-snug" style="height: 30vh;">
                {#each expandedDiffs.get(rev.id) as block}
                  {#if block.text === '\n'}
                    <br />
                  {:else if block.type === 'add'}
                    <span class="text-green-600">{block.text}</span>
                  {:else if block.type === 'del'}
                    <span class="text-red-600 line-through">{block.text}</span>
                  {:else}
                    <span class="text-black dark:text-white">{block.text}</span>
                  {/if}
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  </div>
{:catch reason}
  <span>Could not fetch revisions - {reason}</span>
{/await}
