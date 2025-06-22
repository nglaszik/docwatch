<script lang="ts">
	import { docs } from '$lib/stores/docs';
	import { Button, Dropdown, DropdownItem, ToolbarButton, DropdownDivider, Breadcrumb, BreadcrumbItem } from 'flowbite-svelte';
	import { Table, TableBody, TableBodyCell, TableBodyRow, TableHead, TableHeadCell } from "flowbite-svelte";
	import { Modal } from 'flowbite-svelte';
	import { onMount } from 'svelte';
	import type { Doc } from '$lib/api/docs';
	import { base } from '$app/paths';
	import { page } from "$app/state";
	import { writable } from 'svelte/store';
	
	import SearchOverlay from '$lib/components/SearchOverlay.svelte';
	
	import { DotsHorizontalOutline, FolderOutline, FileDocOutline, ListOutline, GridOutline, PlusOutline } from "flowbite-svelte-icons";
	
	import { deleteDoc, createFolder, editUserDocument } from '$lib/api/docs';
	
	const showSearchOverlay = writable(false);
	
	let showMoveModal = $state(false);
	let movingDocId: string | null = null;
	
	function formatDate(dateStr: string) {
	  const date = new Date(dateStr);
	  return date.toLocaleDateString(undefined, {
		year: 'numeric',
		month: 'short',
		day: 'numeric',
	  });
	}
	
	async function handleCreateFolder() {
		const name = prompt('Enter folder name:');
		if (!name) return;
		await createFolder(name, page.params.folder_id);
		await docs.load(page.params.folder_id);
	}
	
	async function handleDeleteFolder(id: string) {
		await deleteDoc(id);
		await docs.load(page.params.folder_id);
	}
	
	async function handleRename(id: string) {
	  const newName = prompt('Enter new name:');
	  if (!newName) return;
	  try {
		await docs.load(page.params.folder_id); // refresh
	  } catch (err) {
		console.error(err);
		alert('Rename failed.');
	  }
	}
	
	async function handleMove(id: string) {
	  if (id === null) { return };
	  try {
		await editUserDocument(movingDocId, 'id_parent', id);
		showMoveModal = false;
		movingDocId = null;
		await docs.load(page.params.folder_id);
	  } catch (err) {
		console.error(err);
		alert('Move failed.');
	  }
	}
	
	$effect(() => {
		docs.load(page.params.folder_id);
	});
	
</script>

<Modal title="Move to Folder" bind:open={showMoveModal} size="sm">
	<div class="p-4 py-6 overflow-y-auto">
		<Table>
			<TableBody>
				<TableBodyRow>
					<TableBodyCell>
						<div class="flex items-center space-x-2">
							<FolderOutline />
							Home
						</div>
					</TableBodyCell>
					<TableBodyCell>
						<Button onclick={() => handleMove("home")}>Move</Button>
					</TableBodyCell>
				</TableBodyRow>
				{#each $docs.breadcrumbs as folder}
					<TableBodyRow>
						<TableBodyCell>
							<div class="flex items-center space-x-2">
								<FolderOutline />
								<p>{folder.name}</p>
							</div>
						</TableBodyCell>
						<TableBodyCell>
							<Button onclick={() => handleMove(folder.id)}>Move</Button>
						</TableBodyCell>
					</TableBodyRow>
				{/each}
				{#each $docs.docs.filter(d => d.is_folder) as folder}
					<TableBodyRow>
						<TableBodyCell>
							<div class="flex items-center space-x-2">
								<FolderOutline />
								<p>{folder.name}</p>
							</div>
						</TableBodyCell>
						<TableBodyCell>
							<Button onclick={() => handleMove(folder.id)}>Move</Button>
						</TableBodyCell>
					</TableBodyRow>
				{/each}
			</TableBody>
		</Table>
	</div>
</Modal>

{#if $docs.hasLoaded}
<div class="px-4 md:px-6 py-4 md:py-6">
	
	<div class="flex justify-between items-center mb-4">
		
		<Breadcrumb aria-label="Default breadcrumb example">
			<BreadcrumbItem href={`${base}/u/folders/home`} home>Home</BreadcrumbItem>
			{#each $docs.breadcrumbs as breadcrumb}
				<BreadcrumbItem href={`${base}/u/folders/${breadcrumb.id}`}>{breadcrumb.name}</BreadcrumbItem>
			{/each}
		</Breadcrumb>
		
		<div class="flex space-x-2">
			<Button outline={true}><GridOutline class="h-4 w-4" /></Button>
			<Button outline={true}><ListOutline class="h-4 w-4" /></Button>
			<Button outline={true}><PlusOutline class="create-items-button h-4 w-4"/></Button>
			<Dropdown simple triggeredBy=".create-items-button">
				<DropdownItem class="font-medium text-black dark:text-white" onclick={handleCreateFolder}>Create New Folder</DropdownItem>
				<DropdownItem class="font-medium text-black dark:text-white" onclick={() => showSearchOverlay.set(true)}>Add Doc to Watchlist</DropdownItem>
			</Dropdown>
		</div>
		
	</div>
	
	<Table hoverable={true}>
		<TableHead>
			<TableHeadCell>Name</TableHeadCell>
			<TableHeadCell>Owner</TableHeadCell>
			<TableHeadCell>Last Updated</TableHeadCell>
			<TableHeadCell>More</TableHeadCell>
		</TableHead>
		<TableBody>
			{#each $docs.docs as doc}
				{#if doc.is_folder}
					<TableBodyRow>
						<TableBodyCell class="max-w-[150px] whitespace-normal break-words">
							<div class="flex items-center space-x-2">
								<FolderOutline />
								<a href={`${base}/u/folders/${doc.id}`} class="text-black dark:text-white font-medium hover:underline">{doc.name}</a>
							</div>
						</TableBodyCell>
						<TableBodyCell class="max-w-[100px] whitespace-normal break-words"></TableBodyCell>	
						<TableBodyCell class="max-w-[100px] whitespace-normal break-words"></TableBodyCell>			
						<TableBodyCell>
							<DotsHorizontalOutline class={`dots-menu-${doc.id} dark:text-white`} />
							<Dropdown simple triggeredBy={`.dots-menu-${doc.id}`}>
								<DropdownItem class="font-medium text-black dark:text-white" onclick={() => {showMoveModal = true; movingDocId = doc.id;}}>Move</DropdownItem>
								<DropdownItem class="font-medium text-black dark:text-white" onclick={() => handleRename(doc.id)}>Rename</DropdownItem>
								<DropdownItem class="font-medium text-black dark:text-white" onclick={() => handleDeleteFolder(doc.id)}>Delete</DropdownItem>
							</Dropdown>
						</TableBodyCell>
					</TableBodyRow>
				{:else}
					<TableBodyRow>
						<TableBodyCell class="max-w-[150px] whitespace-normal break-words">
							<div class="flex items-center space-x-2">
								<FileDocOutline />
								<a href={`${base}/u/docs/${doc.doc_id}`} class="text-black dark:text-white font-medium hover:underline">{doc.name}</a>
							</div>
						</TableBodyCell>
						<TableBodyCell class="max-w-[100px] whitespace-normal break-words">{doc.owner_username}</TableBodyCell>	
						<TableBodyCell class="max-w-[100px] whitespace-normal break-words">{formatDate(doc.last_updated)}</TableBodyCell>
						<TableBodyCell>
							<DotsHorizontalOutline class={`dots-menu-${doc.id} dark:text-white`} />
							<Dropdown simple triggeredBy={`.dots-menu-${doc.id}`}>
								<DropdownItem class="font-medium text-black dark:text-white" onclick={() => {showMoveModal = true; movingDocId = doc.id;}}>Move</DropdownItem>
								<DropdownItem class="font-medium text-black dark:text-white" onclick={() => handleDeleteFolder(doc.id)}>Remove</DropdownItem>
							</Dropdown>
						</TableBodyCell>
					</TableBodyRow>
				{/if}
			{/each}
		</TableBody>
	</Table>
</div>
{/if}

<SearchOverlay
  open={$showSearchOverlay}
  onClose={() => showSearchOverlay.set(false)}
/>
