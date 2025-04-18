<!-- src/routes/+page.svelte -->
<script lang="ts">
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import { base } from '$app/paths';

  onMount(async () => {
	const res = await fetch(`${base}/api/auth/me`);
	if (res.ok) {
	  goto(`${base}/dashboard`);
	} else {
	  goto(`${base}/login`);
	}
  });
</script>
