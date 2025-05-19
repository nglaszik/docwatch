<!-- src/routes/+page.svelte -->
<script lang="ts">
  // necessary to handle base URL
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { base } from '$app/paths';
  import { user } from '$lib/stores/user';
  onMount(async () => {
    const res = await fetch(`${base}/api/auth/me`);
    if (res.ok) {
      const data = await res.json();
      user.set({ username: data.username });
      goto(`${base}/docs`);
    } else {
      goto(`${base}/login`);
    }
  });
</script>
