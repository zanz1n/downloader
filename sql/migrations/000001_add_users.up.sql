CREATE TYPE "UserRole" AS ENUM ('ADMIN', 'USER');

CREATE TABLE "users" (
    "id" VARCHAR(12) NOT NULL,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "firstName" VARCHAR(16) NOT NULL,
    "lastName" VARCHAR(24) NOT NULL,
    "email" VARCHAR(64) NOT NULL,
    "password" VARCHAR(60) NOT NULL,
    "deleted" BOOLEAN NOT NULL DEFAULT FALSE,
    "role" "UserRole" NOT NULL DEFAULT 'USER',

    CONSTRAINT "users_pkey" PRIMARY KEY ("id")
);

CREATE INDEX "user_email_idx" ON "users"("email");
