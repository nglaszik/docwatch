// src/lib/stores/user.ts
import { writable } from 'svelte/store';

export type User = {
  username: string;
};

export const user = writable<User | null>(null);