// src/lib/api/auth.ts

export async function login(username: string, password: string): Promise<string | null> {
  const res = await fetch('/docwatch/api/auth/login', {
	method: 'POST',
	headers: { 'Content-Type': 'application/json' },
	body: JSON.stringify({ username, password })
  });

  if (res.ok) return null;
  return await res.text();
}

export async function logout(): Promise<void> {
  await fetch('/docwatch/api/auth/logout');
}

export async function checkSession(): Promise<boolean> {
  const res = await fetch('/docwatch/api/auth/me');
  return res.ok;
}
