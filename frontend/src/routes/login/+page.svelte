<script lang="ts">
  import { Input, Button } from 'flowbite-svelte';
  import { login } from '$lib/api/auth';
  import { goto } from '$app/navigation';
  import { base } from '$app/paths';

  let username = '';
  let password = '';
  let error = '';

  async function handleLogin() {
	const err = await login(username, password);
	if (err) {
	  error = err;
	} else {
	  goto(`${base}/dashboard`);
	}
  }
</script>

<div class="max-w-sm mx-auto mt-10">
  <form class="space-y-4" on:submit|preventDefault={handleLogin}>
	<div>
	  <label class="block mb-1 font-medium text-gray-700">Username</label>
	  <Input bind:value={username} placeholder="Enter username" />
	</div>
	<div>
	  <label class="block mb-1 font-medium text-gray-700">Password</label>
	  <Input type="password" bind:value={password} placeholder="Enter password" />
	</div>
	<Button type="submit">Login</Button>
	{#if error}
	  <p class="text-red-600 mt-2">{error}</p>
	{/if}
  </form>
</div>
