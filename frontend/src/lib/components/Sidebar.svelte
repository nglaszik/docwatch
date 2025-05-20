<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { base } from '$app/paths';
  import { page } from '$app/state';
  import { Sidebar, SidebarGroup, SidebarItem, Button } from 'flowbite-svelte';
  import { SearchOutline } from 'flowbite-svelte-icons';
  import { fetchDocs } from '$lib/api/docs';
  import { logout } from '$lib/api/auth';
  import { docs } from '$lib/stores/docs';
  
  let { isOpen = false, closeSidebar = () => {}, openSearchOverlay = () => {} } = $props();
  
  // use this in addition to onMount for triggering reactivity in pages
  let activeUrl = $state(page.url.pathname);
  
  // may want to create an await here too, or put this elsewhere... need to make sure that docs are loaded before revision miniplot is loaded. it's fine here since $docs is reactive anyway
  
  onMount(async () => {
    docs.load();
  });
  
  $effect(() => {
    activeUrl = page.url.pathname;
    docs.load();
  });
  
</script>

<Sidebar
  {activeUrl}
  isOpen={isOpen}
  closeSidebar={closeSidebar}
  backdrop={false}
  activeClass="sidebar-active flex items-center p-2 rounded-lg"
  nonActiveClass="sidebar-inactive flex items-center p-2 rounded-lg"
  position="absolute"
  class="h-full"
>
  <!-- Search -->
  <SidebarGroup>
    <Button onclick={openSearchOverlay}>Add Document</Button>
  </SidebarGroup>

  <!-- Watched Docs -->
  <SidebarGroup label="Watched Docs">
    {#each $docs as doc}
      <SidebarItem
        href={`${base}/docs/${doc.doc_id}`}
        label={doc.name}
      >
      </SidebarItem>
    {/each}
  </SidebarGroup>
</Sidebar>

