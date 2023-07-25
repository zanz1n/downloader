CREATE TABLE "files" (
    "id" VARCHAR(36) NOT NULL,
    "name" VARCHAR(128) NOT NULL,
    "contentType" VARCHAR(64) NOT NULL,
    "size" INTEGER NOT NULL,
    "checksum" VARCHAR(64) NOT NULL,
    "nodeId" VARCHAR(36) NOT NULL,
    "userId" VARCHAR(12) NOT NULL,

    CONSTRAINT "files_pkey" PRIMARY KEY ("id")
);

CREATE INDEX "file_userId_idx" ON "files"("userId");

CREATE INDEX "file_nodeId_idx" ON "files"("nodeId");
