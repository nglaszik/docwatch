<script lang="ts">
  import Sidebar from './Sidebar.svelte';
  import TopNavBar from './TopNavBar.svelte';
  import SearchOverlay from './SearchOverlay.svelte';
  import { writable } from 'svelte/store';
  import { logout } from '$lib/api/auth';
  import { goto } from '$app/navigation';
  import { base } from '$app/paths';
  import { page } from '$app/stores';

  let showSearchOverlay = writable(false);
  let userInitial = 'U';

  function handleLogout() {
    logout().then(() => goto(`${base}/login`));
  }
  
</script>

<div class="flex h-screen">
  
  <Sidebar onShowSearch={() => showSearchOverlay.set(true)} />

  <div class="flex-1 flex flex-col">
    <TopNavBar
      currentDocId={$page.params.doc_id}
      userInitial={userInitial}
      onLogout={handleLogout}
    />

    <main class="flex-1 overflow-auto">
      <slot />
    </main>
  </div>

  <SearchOverlay
    open={$showSearchOverlay}
    onClose={() => showSearchOverlay.set(false)}
  />
</div>
