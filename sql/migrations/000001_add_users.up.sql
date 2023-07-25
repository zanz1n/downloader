CREATE TABLE "users" (
    "id" VARCHAR(12) NOT NULL,
    "firstName" VARCHAR(16) NOT NULL,
    "lastName" VARCHAR(24) NOT NULL,
    "email" VARCHAR(64) NOT NULL,
    "password" VARCHAR(60) NOT NULL,
    "deleted" BOOLEAN NOT NULL,

    CONSTRAINT "users_pkey" PRIMARY KEY ("id")
);

CREATE INDEX "user_email_idx" ON "users"("email");
