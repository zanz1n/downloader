import { z } from "zod";
import { Authenticator } from "./auth";
import { Err, Ok, type Result } from "ts-results-es";
import { AppError } from "./error";
import { safeParse } from "./utils";

const fileDataSchema = z
    .object({
        name: z.string().min(1),
        mime_type: z.string().min(1),
        size: z.number().int().gte(0),
        checksum_256: z.string()
    })
    .transform((o) => ({
        name: o.name,
        mimeType: o.mime_type,
        size: o.size,
        checksum256: o.checksum_256
    }));

export type FileData = z.infer<typeof fileDataSchema>;

const fileSchema = z
    .object({
        id: z.string().uuid(),
        user_id: z.string().uuid(),
        created_at: z
            .string()
            .datetime()
            .transform((v) => new Date(v)),
        updated_at: z
            .string()
            .datetime()
            .transform((v) => new Date(v)),
        data: fileDataSchema
    })
    .transform(
        (o) => new File(o.id, o.user_id, o.created_at, o.updated_at, o.data)
    );

const unitGiB = 1024 * 1024 * 1024;
const unitMiB = 1024 * 1024;
const unitKiB = 1024;

export class File {
    private auth: Authenticator;

    constructor(
        public id: string,
        public userId: string,
        public createdAt: Date,
        public updatedAt: Date,
        public data: FileData
    ) {
        this.auth = Authenticator.getInstance();
    }

    getDownloadUrl(): Result<string, AppError> {
        const url = this.auth.getApiUrl();
        const token = this.auth.getToken();
        if (token.isNone()) {
            return Err(new AppError("Unauthenticated", 401));
        }

        return Ok(`${url}/file/${this.id}/data?token=${token.value}`);
    }

    formatSize(): string {
        let size = this.data.size;

        let unit = "Bytes";
        if (size >= unitGiB) {
            unit = "GiB";
            size /= unitGiB;
        } else if (size >= unitMiB) {
            unit = "MiB";
            size /= unitMiB;
        } else if (size >= unitKiB) {
            unit = "KiB";
            size /= unitKiB;
        }
        size = Math.round(size * 100) / 100;

        console.log(`${size} ${this.data.size}`);

        return size.toString() + " " + unit;
    }
}

const fileSchemaArr = z.array(fileSchema);

export class Files {
    private static INSTANCE: Files;

    private auth: Authenticator;

    private constructor(auth: Authenticator) {
        this.auth = auth;
    }

    static getInstance(): Files {
        if (!this.INSTANCE) {
            const auth = Authenticator.getInstance();
            this.INSTANCE = new Files(auth);
        }

        return this.INSTANCE;
    }

    async getAllFiles(
        limit: number,
        offset = 0
    ): Promise<Result<File[], AppError>> {
        const res = await this.auth.fetch(
            "GET",
            `/file?limit=${limit}&offset=${offset}`
        );
        if (res.isErr()) {
            return res;
        }

        return safeParse(fileSchemaArr, res.value);
    }

    async getUserFiles(
        limit: number,
        offset = 0
    ): Promise<Result<File[], AppError>> {
        const user = this.auth.getAuth();
        if (user.isNone()) {
            return Err(new AppError("Unauthorized"));
        }

        const { userId } = user.value;

        const res = await this.auth.fetch(
            "GET",
            `/file/user/${userId}?limit=${limit}&offset=${offset}`
        );
        if (res.isErr()) {
            return res;
        }

        return safeParse(fileSchemaArr, res.value);
    }

    async getFile(id: string): Promise<Result<File, AppError>> {
        const res = await this.auth.fetch("GET", `/file/${id}`);
        if (!res.isOk()) {
            return res;
        }

        return safeParse(fileSchema, res.value);
    }

    async uploadFile(
        name: string,
        data: Blob
    ): Promise<Result<File, AppError>> {
        const formData = new FormData();
        formData.append("file", data, name);

        const res = await this.auth.fetch("POST", "/file/multipart", formData);
        if (res.isErr()) {
            return res;
        }

        return safeParse(fileSchema, res.value);
    }

    async updateFileData(
        id: string,
        name: string,
        data: Blob
    ): Promise<Result<File, AppError>> {
        const formData = new FormData();
        formData.append("file", data, name);

        const res = await this.auth.fetch(
            "PUT",
            `/file/${id}/multipart`,
            formData
        );
        if (res.isErr()) {
            return res;
        }

        return safeParse(fileSchema, res.value);
    }

    async deleteFile(id: string): Promise<Result<File, AppError>> {
        const res = await this.auth.fetch("DELETE", `/file/${id}`);
        if (!res.isOk()) {
            return res;
        }

        return safeParse(fileSchema, res.value);
    }
}
