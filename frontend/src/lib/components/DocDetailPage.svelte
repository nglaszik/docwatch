<script lang="ts">
  import DashboardLayout from '$lib/components/DashboardLayout.svelte';
  import { fetchRevisions, fetchDocs } from '$lib/api/docs';
  import RevisionMiniPlot from '$lib/components/RevisionMiniPlot.svelte';
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import { writable, get } from 'svelte/store';

  let revisions = [];
  let currentDoc = writable(null);
  let docId = '';

  $: docId = $page.params.doc_id;

  async function loadData() {
	const allDocs = await fetchDocs();
	const doc = allDocs.find(d => d.doc_id === docId);
	if (doc) currentDoc.set(doc);

	revisions = await fetchRevisions(docId);
  }
  
  type DiffBlock = { type: 'add' | 'del' | 'neutral', text: string };
  
  function groupDiffWords(diffJson: string): DiffBlock[] {
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

  onMount(loadData);
</script>

<DashboardLayout bind:currentDoc>
  <div class="p-6 space-y-6">
	{#if $currentDoc}
	  <div class="h-80 bg-gray-100 rounded flex items-center justify-center">
		<RevisionMiniPlot revisions={$currentDoc.revision_summary ?? []} large={true} />
	  </div>

	  <div class="space-y-4">
		{#each revisions as rev}
		  <div class="bg-white border rounded p-4">
			<div class="text-sm font-medium text-gray-800 mb-1">
			  {formatTime(rev.revision_time)} — {rev.added_words} words added, {rev.deleted_words} words deleted
			</div>
			<div class="overflow-y-auto bg-gray-50 border border-gray-300 rounded p-2 text-sm leading-snug" style="height: 30vh;">
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
	{/if}
  </div>
</DashboardLayout>
