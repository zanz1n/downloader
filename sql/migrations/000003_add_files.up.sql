CREATE TABLE
    "files" (
        "id" UUID NOT NULL,
        "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
        "updatedAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
        "name" VARCHAR(128) NOT NULL,
        "contentType" VARCHAR(64) NOT NULL,
        "size" BIGINT NOT NULL,
        "checksum" VARCHAR(64) NOT NULL,
        "nodeId" UUID NOT NULL,
        "userId" UUID NOT NULL,

CONSTRAINT "files_pkey" PRIMARY KEY ("id") );

CREATE INDEX "file_userId_idx" ON "files"("userId");

CREATE INDEX "file_nodeId_idx" ON "files"("nodeId");
