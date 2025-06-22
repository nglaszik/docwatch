// src/lib/api/docs.ts

type Doc = {
  user_doc_id: number;
  doc_id: string | null;
  name: string;
  is_folder: boolean;
  id_parent: number | null;
  last_updated?: string;
  owner_username?: string;
};

export type Breadcrumb = {
  id: string;
  name: string;
};

export type Revision = {
  id: id;
  revision_time: string;
  added_words: number;
  deleted_words: number;
};

export type DiffBlock = {
  type: 'add' | 'del' | 'neutral';
  text: string;
};

export async function fetchDocs(id_parent: string | null = null): Promise<{ docs: Doc[]; breadcrumbs: Breadcrumb[] }> {
  const id = id_parent ?? '';
  const res = await fetch(`/docwatch/api/user_documents/${id}`);
  if (!res.ok) throw new Error('Failed to fetch documents');
  const data = await res.json();
  return data;
}

export async function searchDocs(query: string): Promise<Doc[]> {
  const res = await fetch(`/docwatch/api/docs?q=${encodeURIComponent(query)}`);
  if (!res.ok) throw new Error('Failed to search documents');
  return await res.json();
}

export async function fetchRevisions(docId: string): Promise<Revision[]> {
  const res = await fetch(`/docwatch/api/docs/${docId}/revisions`);
  if (!res.ok) throw new Error('Failed to fetch revisions');
  return await res.json();
}

export async function fetchDiff(revId: number): Promise<DiffBlock[]> {
  const res = await fetch(`/docwatch/api/diffs/${revId}`);
  if (!res.ok) throw new Error('Failed to fetch diff');
  return await res.json();
}

export async function createFolder(name: string, id_parent: string) {
  return await fetch('/docwatch/api/user_documents/create_folder', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name, id_parent }),
  });
}

export async function deleteDoc(user_doc_id: string) {
  await fetch(`/docwatch/api/user_documents/${user_doc_id}`, {
    method: 'DELETE'
  });
}

export async function editUserDocument(
  id: string,
  field: 'id_parent' | 'folder_name',
  value: string
): Promise<void> {
  const res = await fetch('/docwatch/api/user_documents/edit', {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ id, field, value }),
  });

  if (!res.ok) throw new Error(`Failed to update ${field}`);
}

export async function addToWatchlist(docId: string): Promise<void> {
  const res = await fetch('/docwatch/api/docs', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ doc_id: docId })
  });

  if (!res.ok) {
    const err = await res.text();
    console.error(`Failed to add document to watchlist: ${err}`);
    throw new Error(`Failed to add document: ${err}`);
  }
}
