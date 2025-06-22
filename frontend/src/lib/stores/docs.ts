// src/lib/stores/docs.ts
import { writable } from 'svelte/store';
import type { Doc } from '$lib/api/docs';
import type { Breadcrumb } from '$lib/api/docs';
import { fetchDocs } from '$lib/api/docs';

type DocsState = {
  docs: Doc[];
  breadcrumbs: Breadcrumb[];
  hasLoaded: boolean;
};

const { subscribe, set, update } = writable<DocsState>({ docs: [], breadcrumbs: [], hasLoaded: false, });

async function load(id_parent: string | null = null) {
  const data = await fetchDocs(id_parent);
  //set(data);
  set({
    docs: data.docs,
    breadcrumbs: data.breadcrumbs,
    hasLoaded: true,
  });
}

function clear() {
  set({ docs: [], breadcrumbs: [], hasLoaded: false,});
}

export const docs = {
  subscribe,
  load,
  clear
};