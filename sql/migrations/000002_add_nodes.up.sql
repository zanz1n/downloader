CREATE TABLE "nodes" (
    "id" VARCHAR(36) NOT NULL,
    "name" VARCHAR(64) NOT NULL,
    "description" TEXT NOT NULL,
    "address" VARCHAR(128) NOT NULL,
    "ssl" BOOLEAN NOT NULL,
    "capacity" INTEGER NOT NULL,

    CONSTRAINT "nodes_pkey" PRIMARY KEY ("id")
);
