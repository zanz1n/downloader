import { Err, None, Ok, Result, Some, type Option } from "ts-results-es";
import { z } from "zod";
import { userSchema, type User } from "./user";
import { jwtDecode } from "jwt-decode";
import { AppError, appErrorSchema } from "./error";

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
        perm: z.number().int().nonnegative(),
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
            this.INSTANCE = new Authenticator("/api");
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

    getAuth(): Option<Auth> {
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

    logout() {
        try {
            this.removeAuthToken();
        } catch (e) {
            console.error("Authenticator.logout gone wrong:", e);
        }
    }

    async getUser(): Promise<Result<User, AppError>> {
        try {
            const res = await this.fetch("/user/self", null);
            if (!res.isOk()) {
                return Err(res.error);
            }

            const json = await res.value.json();
            if (!res.value.ok) {
                const error = appErrorSchema.parse(json);
                return Err(error);
            }

            const user = userSchema.parse(json);
            return Ok(user);
        } catch (e) {
            if (e instanceof Error) {
                return Err(new AppError(e.message, 0));
            }
            return Err(new AppError("Unknown", 0));
        }
    }

    async login(data: LoginData): Promise<Result<User, AppError>> {
        try {
            const res = await fetch(this.url + "/auth/login", {
                body: JSON.stringify(data),
                method: "POST",
                headers: {
                    Accept: "application/json",
                    "Content-Type": "application/json"
                }
            });

            const json = await res.json();
            if (!res.ok) {
                const error = appErrorSchema.parse(json);
                return Err(error);
            }

            const resData = loggedInSchema.parse(json);
            this.setAuthToken(resData.token);

            return Ok(resData.user);
        } catch (e) {
            if (e instanceof Error) {
                return Err(new AppError(e.message, 0));
            }
            return Err(new AppError("Unknown", 0));
        }
    }

    async signup(
        data: LoginData,
        serverKey: string
    ): Promise<Result<User, AppError>> {
        try {
            const res = await fetch(this.url + "/auth/signup", {
                body: JSON.stringify(data),
                method: "POST",
                headers: {
                    Accept: "application/json",
                    "Content-Type": "application/json",
                    Authorization: "Secret " + serverKey
                }
            });

            const json = await res.json();
            if (!res.ok) {
                const error = appErrorSchema.parse(json);
                return Err(error);
            }

            const resData = loggedInSchema.parse(json);
            this.setAuthToken(resData.token);

            return Ok(resData.user);
        } catch (e) {
            if (e instanceof Error) {
                return Err(new AppError(e.message, 0));
            }
            return Err(new AppError("Unknown", 0));
        }
    }

    async fetch(
        input: string,
        data: unknown
    ): Promise<Result<Response, AppError>> {
        try {
            const token = this.getAuthToken();
            if (!token) {
                return Err(new AppError("Unauthorized", 0));
            }

            const headers = {
                Accept: "application/json",
                Authorization: "Bearer " + token
            } as Record<string, string>;

            let body = null;
            if (data) {
                headers["Content-Type"] = "application/json";
                body = JSON.stringify(data);
            }

            const res = await fetch(this.url + input, {
                body,
                headers
            });

            return Ok(res);
        } catch (e) {
            if (e instanceof Error) {
                return Err(new AppError(e.message, 0));
            }
            return Err(new AppError("Unknown", 0));
        }
    }
}
