CREATE TABLE "nodes" (
    "id" VARCHAR(36) NOT NULL,
    "createdAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "name" VARCHAR(64) NOT NULL,
    "description" TEXT NOT NULL,
    "address" VARCHAR(128) NOT NULL,
    "port" INTEGER NOT NULL,
    "tcp" BOOLEAN NOT NULL,
    "tcpPort" INTEGER,
    "ssl" BOOLEAN NOT NULL,
    "capacity" BIGINT NOT NULL,

    CONSTRAINT "nodes_pkey" PRIMARY KEY ("id")
);
