// src/lib/stores/docs.ts
import { writable } from 'svelte/store';
import type { Doc } from '$lib/api/docs';
import { fetchDocs } from '$lib/api/docs';

const { subscribe, set } = writable<Doc[]>([]);

let hasLoaded = false;

async function load(force = false) {
  if (hasLoaded && !force) return;
  const data = await fetchDocs();
  set(data);
  hasLoaded = true;
}

function clear() {
  set([]);
  hasLoaded = false;
}

export const docs = {
  subscribe,
  load,
  clear
};
