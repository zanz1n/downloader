import { Authenticator } from "$lib/auth.js";

export const prerender = false;
export const csr = true;
export const ssr = false;

export async function load() {
    const auth = Authenticator.getInstance().getAuth();
    return { auth };
}
