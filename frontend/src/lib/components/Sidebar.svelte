<script lang="ts">
  
  import { base } from '$app/paths';
  import { Sidebar, SidebarGroup, SidebarItem, Button } from 'flowbite-svelte';
  import { FolderOutline, ChartMixedOutline } from "flowbite-svelte-icons";
  
  let { isOpen = false, closeSidebar = () => {} } = $props();
  
</script>

<Sidebar
  isOpen={isOpen}
  closeSidebar={closeSidebar}
  backdrop={false}
  activeClass="sidebar-active flex items-center p-2 rounded-md"
  nonActiveClass="sidebar-inactive flex items-center p-2 rounded-md"
  position="absolute"
  class="h-full w-48 py-2"
>
  <!-- Search -->
  <SidebarGroup>
    <SidebarItem label="Menu"></SidebarItem>
    <SidebarItem href={`${base}/u/folders/home`} label="Folders">
      {#snippet icon()}
        <FolderOutline />
      {/snippet}
    </SidebarItem>
    <SidebarItem label="Reports">
      {#snippet icon()}
        <ChartMixedOutline />
      {/snippet}
    </SidebarItem>
  </SidebarGroup>
</Sidebar>

