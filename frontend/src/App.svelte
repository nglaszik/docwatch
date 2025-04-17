<script lang="ts">
  import { onMount } from "svelte";
  import { Table, Button, Input } from 'flowbite-svelte';

  let username = "";
  let password = "";
  let error = "";
  let loggedIn = false;

  let newDocId = "";
  let docs = [];
  
  let visibleRevisions = new Map();  // doc_id -> revision array
  let expandedDocs = new Set(); // holds doc_ids that are currently expanded

  async function login() {
    const res = await fetch("/api/auth/login", {
      method: "POST",
      headers: {
        "Content-Type": "application/json"
      },
      body: JSON.stringify({ username, password })
    });

    if (res.ok) {
      loggedIn = true;
      error = "";
      await fetchDocs(); // ✅ only fetch after login
    } else {
      error = await res.text();
      loggedIn = false;
    }
  }
  
  async function logout() {
    try {
      const res = await fetch("/api/auth/logout");
      if (res.ok) {
        loggedIn = false;
        docs = [];
        visibleRevisions.clear();
      } else {
        console.error("Logout failed");
      }
    } catch (err) {
      console.error("Logout error:", err);
    }
  }

  async function fetchDocs() {
    const res = await fetch("/api/docs");
    docs = await res.json();
    console.log("Fetched docs:", docs);
  }

  async function addDoc() {
    await fetch("/api/docs", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ doc_id: newDocId })
    });
    newDocId = "";
    await fetchDocs(); // refresh the list
  }
  
  function formatTime(iso) {
    const d = new Date(iso);
    return d.toLocaleString();
  }
  
  async function fetchRevisions(docId) {
    // If we've never fetched it, fetch it first
    if (!visibleRevisions.has(docId)) {
      console.log("📡 Fetching revisions for", docId);
      const res = await fetch(`/api/docs/${docId}/revisions`);
      const data = await res.json();
      visibleRevisions.set(docId, data);
    }
  
    // Always toggle visibility
    if (expandedDocs.has(docId)) {
      expandedDocs.delete(docId);
    } else {
      expandedDocs.add(docId);
    }
  
    // Trigger Svelte reactivity
    visibleRevisions = new Map(visibleRevisions);
    expandedDocs = new Set(expandedDocs);
  }

  
  onMount(async () => {
    
    const res = await fetch("/api/auth/me");
    
    if (res.ok) {
      loggedIn = true;
      fetchDocs();
      
      //const interval = setInterval(fetchDocs, 60000); // refresh every 60 seconds
      
      //return () => {
      //  clearInterval(interval); // clean up if component is destroyed
      //};
      
    }
    
  });
  
</script>

{#if !loggedIn}
  <div class="max-w-sm mx-auto mt-10">
    <form class="space-y-4" on:submit|preventDefault={login}>
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
{:else}
  <div class="flex h-screen">
    <!-- Sidebar -->
    <div class="w-64 bg-gray-100 p-4 border-r border-gray-300">
      <h1 class="text-2xl font-bold mb-6">📄 Monitored Docs</h1>
      <form on:submit|preventDefault={addDoc} class="space-y-4">
        <Input bind:value={newDocId} placeholder="Google Doc ID" />
        <Button type="submit" color="blue">Add Document</Button>
        <Button color="light" on:click={logout}>Logout</Button>
      </form>
    </div>

    <!-- Main Content -->
    <div class="flex-1 p-6 overflow-auto">
      <Table>
        <table class="w-full text-sm text-left text-gray-500">
          <thead class="text-xs text-gray-700 uppercase bg-gray-50">
            <tr>
              <th scope="col" class="px-6 py-3">Document</th>
              <th scope="col" class="px-6 py-3">Last Updated</th>
            </tr>
          </thead>
          <tbody>
            {#each docs as doc}
              <tr class="bg-white border-b">
                <td class="px-6 py-4">
                  <strong>{doc.name}</strong><br />
                  <Button size="xs" on:click={() => fetchRevisions(doc.doc_id)} color="blue" class="mt-1">
                    {expandedDocs.has(doc.doc_id) ? "Hide Revisions" : "View Revisions"}
                  </Button>
                </td>
                <td class="px-6 py-4">{formatTime(doc.last_updated)}</td>
              </tr>
      
              {#if expandedDocs.has(doc.doc_id)}
                <tr>
                  <td colspan="2">
                    <h4 class="text-md font-semibold mb-2">Revision History</h4>
                    <ul class="space-y-3">
                      {#each visibleRevisions.get(doc.doc_id) as rev}
                        <li>
                          <div class="text-sm font-medium text-gray-800 mb-1">
                            {formatTime(rev.revision_time)} — {rev.added_chars} chars added
                          </div>
                          <pre class="whitespace-pre-wrap break-words overflow-x-auto bg-gray-50 border border-gray-300 rounded p-2 text-sm">
                            {rev.diff}
                          </pre>
                        </li>
                      {/each}
                    </ul>
                  </td>
                </tr>
              {/if}

            {/each}
          </tbody>
        </table>
      </Table>
    </div>
  </div>
{/if}
