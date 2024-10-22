import { None, Some, type Option } from "ts-results-es";
import { z } from "zod";
import { userSchema } from "./user";
import { jwtDecode } from "jwt-decode";

export const authSchema = z
    .object({
        sub: z.string().uuid(),
        iat: z
            .number()
            .int()
            .nonnegative()
            .transform((v) => new Date(v * 1000)),
        exp: z
            .number()
            .int()
            .nonnegative()
            .transform((v) => new Date(v * 1000)),
        iss: z.string(),
        perm: z.number().nonnegative(),
        username: z.string().min(1)
    })
    .transform((o) => ({
        userId: o.sub,
        username: o.username,
        createdAt: o.iat,
        expiresAt: o.exp,
        issuer: o.iss,
        permission: o.perm
    }));

export type Auth = z.infer<typeof authSchema>;

export type LoginData = {
    username: string;
    password: string;
};

export const loggedInSchema = z.object({
    user: userSchema,
    token: z.string().min(1)
});

export class Authenticator {
    private url: string;

    private static INSTANCE: Authenticator | undefined;

    constructor(url: string) {
        this.url = url;
    }

    static getInstance(): Authenticator {
        if (!this.INSTANCE) {
            this.INSTANCE = new Authenticator("/");
        }

        return this.INSTANCE;
    }

    private getAuthToken(): string | null {
        return localStorage.getItem("auth_token");
    }

    private setAuthToken(token: string) {
        return localStorage.setItem("auth_token", token);
    }

    private removeAuthToken() {
        return localStorage.removeItem("auth_token");
    }

    async getAuth(): Promise<Option<Auth>> {
        try {
            const dataS = this.getAuthToken();
            if (!dataS) {
                return None;
            }

            const data = authSchema.parse(jwtDecode(dataS));
            if (new Date() >= data.expiresAt) {
                this.removeAuthToken();
                return None;
            }

            return Some(data);
        } catch (e) {
            console.error("Authenticator.getAuth gone wrong:", e);
            return None;
        }
    }
}
