<script lang="ts">
  import { Dropdown, Avatar } from 'flowbite-svelte';
  import { page } from '$app/stores';
  import { fetchDocs } from '$lib/api/docs';
  import { onMount } from 'svelte';

  export let userInitial = 'U';
  export let onLogout = () => {};

  let currentDocName = 'Select a Document';

  async function loadDocName() {
    const docId = $page.params.doc_id;
    if (!docId) return;

    const docs = await fetchDocs();
    const doc = docs.find(d => d.doc_id === docId);
    if (doc) {
      currentDocName = doc.name;
    }
  }

  onMount(loadDocName);
  $: if ($page.params.doc_id) loadDocName();
</script>

<header class="h-14 bg-white border-b border-gray-300 flex items-center justify-between px-6">
  <h2 class="text-lg font-semibold">{currentDocName}</h2>
  <Dropdown placement="bottom-end">
    <Avatar rounded size="md" placeholder={userInitial} slot="trigger" class="cursor-pointer" />
    <div class="py-1">
      <button class="w-full text-left px-4 py-2 hover:bg-gray-100" on:click={onLogout}>Logout</button>
    </div>
  </Dropdown>
</header>
