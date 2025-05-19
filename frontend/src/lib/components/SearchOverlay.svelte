<script lang="ts">
  import { Modal, Input, Button } from 'flowbite-svelte';
  import { searchDocs, addToWatchlist } from '$lib/api/docs';
  import { docs } from '$lib/stores/docs';

  let { open = false, onClose = () => {} } = $props();

  let query = $state('');
  let results = $state([]);

  async function handleSearch() {
    results = await searchDocs(query);
  }
  
  async function handleAdd(docId: string) {
    await addToWatchlist(docId);
    await docs.load(true); // added this!
  }
  
  $effect(() => {
    
    if (query.length >= 2) {
      handleSearch();
    } else {
      results = [];
    }
    
    if (!open) {
      onClose();
    }
    
  });

</script>

<Modal title="Search Documents" bind:open size="lg">
  <!-- Search Input pinned at top -->
  <div class="mb-4">
    <Input bind:value={query} placeholder="Search documents or users..." />
  </div>

  <!-- Scrollable area with fixed height -->
  <div class="h-96 overflow-y-auto space-y-4 pr-2">
    {#each results as doc}
      <div class="p-4 rounded shadow">
        <h3 class="text-lg font-semibold">{doc.name}</h3>
        <p class="text-sm">ID: {doc.doc_id}</p>
        <p class="text-sm">Owner: {doc.owner_username}</p>
        <Button size="xs" color="primary" class="mt-2" onclick={() => handleAdd(doc.doc_id)}>Watch</Button>
      </div>
    {/each}
  
    {#if query.length >= 2 && results.length === 0}
      <p class="text-center text-gray-500">No results found.</p>
    {/if}
  </div>
</Modal>
