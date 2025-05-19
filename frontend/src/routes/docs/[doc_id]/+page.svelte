<script lang="ts">
  
  import { fetchRevisions } from '$lib/api/docs';
  import RevisionMiniPlot from '$lib/components/RevisionMiniPlot.svelte';
  import { page } from '$app/stores';
  import { docs } from '$lib/stores/docs';
  import { get } from 'svelte/store';
  
  // revisions reacts whenever doc_id changes, then html awaits response to render
  const revisions = $derived(fetchRevisions($page.params.doc_id));
  
  type DiffBlock = { type: 'add' | 'del' | 'neutral', text: string };
  
  function groupDiffWords(diffJson: string): DiffBlock[] {
    
      if (!diffJson || diffJson.trim() === "") {
        return [];
      }
    
    try {
      const parsed = JSON.parse(diffJson);
      if (!Array.isArray(parsed)) return [];
  
      return parsed.map((entry: { type: string; text: string }) => {
        switch (entry.type) {
          case 'Added': return { type: 'add', text: entry.text };
          case 'Removed': return { type: 'del', text: entry.text };
          case 'Unchanged': return { type: 'neutral', text: entry.text };
          default: return { type: 'neutral', text: entry.text };
        }
      });
    } catch (e) {
      console.error("Failed to parse diff JSON:", e);
      return [];
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
    <div class="w-full max-w-screen-lg space-y-6">
      <div class="w-full h-80 rounded flex items-center justify-center">
        <RevisionMiniPlot docId={$page.params.doc_id ?? ''} large={true} />
      </div>
      <div class="w-full mx-auto p-4 space-y-4">
        {#each revisions as rev}
          <div class="border rounded p-4">
            <div class="text-sm font-medium mb-1">
              {formatTime(rev.revision_time)} — {rev.added_words} words added, {rev.deleted_words} words deleted
            </div>
            <div class="overflow-y-auto border border-gray-300 rounded p-2 text-sm leading-snug" style="height: 30vh;">
              {#each groupDiffWords(rev.diff) as block}
                {#if block.text === '\n'}
                  <br />
                {:else if block.type === 'add'}
                  <span class="text-green-600">{block.text}</span>
                {:else if block.type === 'del'}
                  <span class="text-red-600 line-through">{block.text}</span>
                {:else}
                  <span>{block.text}</span>
                {/if}
              {/each}
            </div>
          </div>
        {/each}
      </div>
    </div>
  </div>
{:catch reason}
  <span>Could not fetch revisions - {reason}</span>
{/await}
