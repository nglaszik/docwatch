<script lang="ts">
  import { onMount } from 'svelte';
  import { searchDocs, addToWatchlist } from '$lib/api/docs';

  let query = '';
  let results = [];
  let showOverlay = false;

  const debounce = (fn, delay) => {
	let timeout;
	return (...args) => {
	  clearTimeout(timeout);
	  timeout = setTimeout(() => fn(...args), delay);
	};
  };

  const handleSearch = debounce(async () => {
	if (query.length < 2) {
	  results = [];
	  return;
	}
	results = await searchDocs(query);
  }, 300);

  function toggleOverlay() {
	showOverlay = !showOverlay;
	if (!showOverlay) query = '';
  }
</script>

<style>
  .overlay {
	@apply fixed inset-0 bg-white bg-opacity-90 z-50 overflow-auto;
  }
</style>

<!-- Floating Search Button -->
<button
  class="fixed top-4 right-4 z-40 btn btn-outline btn-circle"
  on:click={toggleOverlay}
>
  🔍
</button>

{#if showOverlay}
  <div class="overlay">
	<div class="p-4 max-w-3xl mx-auto">
	  <div class="flex justify-between items-center mb-4">
		<input
		  type="text"
		  placeholder="Search documents or users..."
		  class="input input-bordered w-full"
		  bind:value={query}
		  on:input={handleSearch}
		  autofocus
		/>
		<button on:click={toggleOverlay} class="btn btn-ghost ml-2">Close</button>
	  </div>

	  <div class="space-y-4">
		{#each results as doc}
		  <div class="card bg-gray-100 p-4 shadow rounded">
			<h3 class="text-lg font-semibold">{doc.name}</h3>
			<p class="text-sm">ID: {doc.doc_id}</p>
			<p class="text-sm">Owner: {doc.owner_username}</p>
			<button class="btn btn-primary mt-2" on:click={() => addToWatchlist(doc.doc_id)}>Watch</button>
		  </div>
		{/each}
		{#if query.length >= 2 && results.length === 0}
		  <p class="text-center text-gray-500">No results found.</p>
		{/if}
	  </div>
	</div>
  </div>
{/if}
