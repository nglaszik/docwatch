<script lang="ts">
  import { Input, Button } from 'flowbite-svelte';
  import { login } from '$lib/api/auth';
  import { goto } from '$app/navigation';
  import { base } from '$app/paths';
  
  import { user } from '$lib/stores/user';

  let username = '';
  let password = '';
  let error = '';

  async function handleLogin() {
	const err = await login(username, password);
    if (err) {
      error = err;
    } else {
      const res = await fetch(`${base}/api/auth/me`);
      if (res.ok) {
        const data = await res.json();
        user.set({ username: data.username });
      }
      goto(`${base}/docs`);
    }
  }
</script>

<div class="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-100 via-white to-gray-200">
  <div class="backdrop-blur-md bg-white/70 border border-gray-200 shadow-xl rounded-xl p-8 w-full max-w-sm">
    <h1 class="text-3xl font-bold text-center mb-6">Login</h1>
    <form class="space-y-4" on:submit|preventDefault={handleLogin}>
      <div>
        <label for="username" class="block mb-1 font-medium text-gray-700">Username</label>
        <Input id="username" bind:value={username} placeholder="Enter username" />
      </div>
      <div>
        <label for="password" class="block mb-1 font-medium text-gray-700">Password</label>
        <Input id="password" type="password" bind:value={password} placeholder="Enter password" />
      </div>
      <Button type="submit" color="primary" class="w-full">Login</Button>
        {#if error}
          <p class="text-red-600 mt-2 text-center">{error}</p>
        {/if}
    </form>
  </div>
</div>
