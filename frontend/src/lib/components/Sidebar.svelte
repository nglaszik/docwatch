<script lang="ts">
  import { Button } from 'flowbite-svelte';
  import { fetchDocs } from '$lib/api/docs';
  import { logout } from '$lib/api/auth';
  import { goto } from '$app/navigation';
  import { base } from '$app/paths';
  import { page } from '$app/stores';

  export let onShowSearch = () => {};

  let docs = [];

  async function fetchDocsHandler() {
    docs = await fetchDocs();
  }

  async function handleLogout() {
    await logout();
    goto(`${base}/login`);
  }

  fetchDocsHandler();
</script>

<aside class="w-64 border-r border-gray-300 flex flex-col">
  <div class="flex items-center justify-between p-4">
    <h1 class="text-xl font-bold">Watched Docs</h1>
    <Button size="xs" color="light" onclick={onShowSearch}>🔍</Button>
  </div>
  <nav class="flex-1 overflow-auto">
    {#each docs as doc}
      <button
        class="w-full text-left px-4 py-2 hover:bg-gray-200"
        class:bg-gray-300={doc.doc_id === $page.params.doc_id}
        class:font-semibold={doc.doc_id === $page.params.doc_id}
        on:click={() => goto(`${base}/doc/${doc.doc_id}`)}
      >
        {doc.name}
      </button>
    {/each}
  </nav>
  <div class="p-4">
    <Button color="primary" size="sm" onclick={handleLogout}>Logout</Button>
  </div>
</aside>
