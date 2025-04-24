<script lang="ts">
  import { onMount } from 'svelte';
  import { Input, Button, Table } from 'flowbite-svelte';
  import { goto } from '$app/navigation';
  import { base } from '$app/paths';
  import { logout, checkSession } from '$lib/api/auth';

  import { fetchDocs, addDoc, fetchRevisions } from '$lib/api/docs';
  import type { Doc, Revision } from '$lib/api/docs';
  
  import RevisionMiniPlot from '$lib/RevisionMiniPlot.svelte';
  
  let docs: Doc[] = [];
  
  let newDocId = '';
  let visibleRevisions = new Map();
  let expandedDocs = new Set();
  
  type DiffBlock = { type: 'add' | 'del' | 'neutral', text: string };
  
  export function groupDiffWords(diffJson: string): DiffBlock[] {
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

  async function addDocHandler() {
	await addDoc(newDocId);
	newDocId = '';
	docs = await fetchDocs();
  }
  
  async function fetchDocsHandler() {
	docs = await fetchDocs();
  }
  
  async function fetchRevisionsHandler(docId: string) {
	if (!visibleRevisions.has(docId)) {
	  const revisions = await fetchRevisions(docId);
	  visibleRevisions.set(docId, revisions);
	}
  
	if (expandedDocs.has(docId)) {
	  expandedDocs.delete(docId);
	} else {
	  expandedDocs.add(docId);
	}
  
	// trigger reactivity
	visibleRevisions = new Map(visibleRevisions);
	expandedDocs = new Set(expandedDocs);
  }

  function formatTime(iso) {
	const d = new Date(iso);
	return d.toLocaleString();
  }
  
  async function handleLogout() {
	await logout();
	goto(`${base}/login`);
  }

  onMount(async () => {
	const ok = await checkSession();
	if (!ok) goto(`${base}/login`);
	else await fetchDocsHandler();
  });
  
</script>

<div class="flex h-screen">
  <!-- Sidebar -->
  <div class="w-64 bg-gray-100 p-4 border-r border-gray-300">
	<h1 class="text-2xl font-bold mb-6">📄 Monitored Docs</h1>
	<form on:submit|preventDefault={addDoc} class="space-y-4">
	  <Input bind:value={newDocId} placeholder="Google Doc ID" />
	  <Button type="submit" color="blue">Add Document</Button>
	  <Button color="light" on:click={logout}>Logout</Button>
	</form>
  </div>

  <!-- Main Content -->
  <div class="flex-1 p-6 overflow-auto">
	<Table>
	  <table class="w-full text-sm text-left text-gray-500">
		<thead class="text-xs text-gray-700 uppercase bg-gray-50">
		  <tr>
			<th scope="col" class="px-6 py-3">Name</th>
			<th scope="col" class="px-6 py-3">Revisions Plot</th>
			<th scope="col" class="px-6 py-3">Last Updated</th>
			<th scope="col" class="px-6 py-3">Expand</th>
		  </tr>
		</thead>
		<tbody>
		  {#each docs as doc}
			<tr class="bg-white border-b">
			  <td class="px-6 py-4"><strong>{doc.name}</strong><br /></td>
			  <td class="px-6 py-4"><RevisionMiniPlot revisions={doc.revision_summary ?? []} /></td>
			  <td class="px-6 py-4">{formatTime(doc.last_updated)}</td>
			  <td class="px-6 py-4">
				<Button size="xs" on:click={() => fetchRevisionsHandler(doc.doc_id)} color="blue" class="mt-1">
				  {expandedDocs.has(doc.doc_id) ? "Hide Revisions" : "View Revisions"}
			    </Button>
		  	  </td>
			</tr>

			{#if expandedDocs.has(doc.doc_id)}
			  <tr>
				<td colspan="2">
				  <h4 class="text-md font-semibold mb-2">Revision History</h4>
				  <ul class="space-y-3">
					{#each visibleRevisions.get(doc.doc_id) as rev}
					  <li>
						<div class="text-sm font-medium text-gray-800 mb-1">
						  {formatTime(rev.revision_time)} — {rev.added_words} words added
						</div>
						<div class="whitespace-pre-wrap break-words overflow-x-auto bg-gray-50 border border-gray-300 rounded p-2 text-sm leading-snug">
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
					  </li>
					{/each}
				  </ul>
				</td>
			  </tr>
			{/if}
		  {/each}
		</tbody>
	  </table>
	</Table>
  </div>
</div>
