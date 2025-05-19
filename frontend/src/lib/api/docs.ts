// src/lib/api/docs.ts

export type RevisionSummary = {
  revision_time: string;
  added_chars: number;
  deleted_chars: number;
};

export type Doc = {
  doc_id: string;
  name: string;
  last_updated: string;
  revision_summary: RevisionSummary[];
  owner_username: string;
};

export type Revision = {
  revision_time: string;
  added_chars: number;
  deleted_chars: number;
  diff: string;
};

export async function fetchDocs(): Promise<Doc[]> {
  const res = await fetch('/docwatch/api/docs');
  if (!res.ok) throw new Error('Failed to fetch documents');
  return await res.json();
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
