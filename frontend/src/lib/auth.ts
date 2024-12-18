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

    private constructor(url: string) {
        this.url = url;
    }

    static getInstance(): Authenticator {
        if (!this.INSTANCE) {
            this.INSTANCE = new Authenticator("/api");
        }

        return this.INSTANCE;
    }

    private _getAuthToken(): string | null {
        return localStorage.getItem("auth_token");
    }

    private _setAuthToken(token: string) {
        return localStorage.setItem("auth_token", token);
    }

    private _removeAuthToken() {
        return localStorage.removeItem("auth_token");
    }

    getApiUrl(): string {
        return this.url;
    }

    getToken(): Option<string> {
        const tk = this._getAuthToken();
        if (tk) {
            return Some(tk);
        }
        return None;
    }

    getAuth(): Option<Auth> {
        try {
            const dataS = this._getAuthToken();
            if (!dataS) {
                return None;
            }

            const data = authSchema.parse(jwtDecode(dataS));
            if (new Date() >= data.expiresAt) {
                this._removeAuthToken();
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
            this._removeAuthToken();
        } catch (e) {
            console.error("Authenticator.logout gone wrong:", e);
        }
    }

    async getUser(): Promise<Result<User, AppError>> {
        try {
            const res = await this.fetch("GET", "/user/self", null);
            if (!res.isOk()) {
                return Err(res.error);
            }

            const user = userSchema.parse(await res.value.json());
            return Ok(user);
        } catch (e) {
            if (e instanceof Error) {
                return Err(new AppError(e.message));
            }
            return Err(new AppError("Unknown"));
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
            this._setAuthToken(resData.token);

            return Ok(resData.user);
        } catch (e) {
            if (e instanceof Error) {
                return Err(new AppError(e.message));
            }
            return Err(new AppError("Unknown"));
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
            this._setAuthToken(resData.token);

            return Ok(resData.user);
        } catch (e) {
            if (e instanceof Error) {
                return Err(new AppError(e.message));
            }
            return Err(new AppError("Unknown"));
        }
    }

    async changePassword(
        oldpass: string,
        newpass: string
    ): Promise<Result<User, AppError>> {
        try {
            const token = this.getAuth();
            if (token.isNone()) {
                return Err(new AppError("Unauthenticated", 401));
            }

            const data = {
                username: token.value.username,
                old_password: oldpass,
                new_password: newpass
            };

            const res = await fetch(this.url + "/auth/password", {
                body: JSON.stringify(data),
                method: "PUT",
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
            this._setAuthToken(resData.token);

            return Ok(resData.user);
        } catch (e) {
            if (e instanceof Error) {
                return Err(new AppError(e.message));
            }
            return Err(new AppError("Unknown"));
        }
    }

    async fetch(
        method: string,
        input: string,
        data?: unknown
    ): Promise<Result<Response, AppError>> {
        try {
            const token = this._getAuthToken();
            if (!token) {
                return Err(new AppError("Unauthenticated", 401));
            }

            const headers = {
                Accept: "application/json",
                Authorization: "Bearer " + token
            } as Record<string, string>;

            let body = null;
            if (data) {
                if (data instanceof FormData) {
                    body = data;
                } else if (data instanceof Blob) {
                    body = data;
                    headers["Content-Type"] = data.type;
                } else {
                    headers["Content-Type"] = "application/json";
                    body = JSON.stringify(data);
                }
            }

            const res = await fetch(this.url + input, {
                body,
                headers,
                method
            });

            if (!res.ok) {
                const json = await res.json();
                const err = appErrorSchema.parse(json);
                return Err(err);
            }

            return Ok(res);
        } catch (e) {
            if (e instanceof Error) {
                return Err(new AppError(e.message));
            }
            return Err(new AppError("Unknown"));
        }
    }
}
