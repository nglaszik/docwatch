<!-- src/routes/docs/+layout.svelte -->
<script lang="ts">
  
  import { onMount } from 'svelte';
  
  import { Navbar, NavBrand, NavLi, NavUl, NavHamburger, Avatar, Dropdown, DropdownItem, DropdownHeader, DropdownGroup, uiHelpers, SidebarButton, DarkMode } from "flowbite-svelte";
  
  import Sidebar from '$lib/components/Sidebar.svelte';
  import SearchOverlay from '$lib/components/SearchOverlay.svelte';
  
  import { writable, derived } from 'svelte/store';
  import { logout } from '$lib/api/auth';
  import { goto } from '$app/navigation';
  import { base } from '$app/paths';
  import { page } from "$app/state";
  
  import { user } from '$lib/stores/user';
  
  const sidebarUi = uiHelpers();
  let isOpen = $state(sidebarUi.isOpen);
  let initial = $state('');
  let username = $state('');
  
  // reactivity for user store and sidebar... this reacts to $user changing I believe... or maybe all of them? whatever
  $effect(() => {
    console.log('main page reactivity triggered');
    isOpen = sidebarUi.isOpen;
    username = $user?.username ?? '';
    initial = $user?.username?.charAt(0).toUpperCase() ?? '';
  });

  const showSearchOverlay = writable(false);

  function handleLogout() {
    logout().then(() => goto(`${base}/login`));
  }
  
  onMount(async () => {
    const res = await fetch(`${base}/api/auth/me`);
    if (res.ok) {
      const data = await res.json();
      user.set({ username: data.username });
    } else {
      goto(`${base}/login`);
    }
  });
  
</script>

<Navbar>
  <NavBrand href="/docwatch">
    <img src="/docwatch/docwatch_icon.svg" class="me-3 h-6 sm:h-9" alt="Docwatch Logo" />
    <span class="self-center text-xl font-semibold whitespace-nowrap dark:text-white">Docwatch</span>
  </NavBrand>
  <div class="ml-auto flex items-center gap-4">
    <DarkMode class="text-primary-500 dark:text-primary-600 border dark:border-gray-800" />
    <Avatar id="avatar-menu">
      <span class="select-none cursor-pointer">{initial}</span>
    </Avatar>
    <Dropdown placement="bottom" triggeredBy="#avatar-menu" simple>
      <DropdownHeader>
        <span class="block truncate text-sm font-medium">{username}</span>
      </DropdownHeader>
      <DropdownItem onclick={handleLogout}>Sign out</DropdownItem>
    </Dropdown>
  </div>
</Navbar>

<SidebarButton onclick={sidebarUi.toggle} class="mb-4" />

<div class="relative">
  
  <Sidebar
    isOpen={isOpen}
    closeSidebar={sidebarUi.close}
    openSearchOverlay={() => showSearchOverlay.set(true)}
  />

  <div class="flex h-full">
    <div class="flex-1 flex flex-col">
      <main class="flex-1 overflow-auto md:ml-64">
        <slot />
      </main>
    </div>
  </div>
</div>

<!-- Search Overlay modal (flows above everything else) -->
<SearchOverlay
  open={$showSearchOverlay}
  onClose={() => showSearchOverlay.set(false)}
/>