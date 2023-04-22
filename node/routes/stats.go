package routes

import (
	"runtime"
	"syscall"

	"github.com/gofiber/fiber/v2"
)

func GetStats() func(c *fiber.Ctx) error {
	return func(c *fiber.Ctx) error {
		info := &syscall.Sysinfo_t{}

		if err := syscall.Sysinfo(info); err != nil {
			return c.Status(fiber.StatusInternalServerError).JSON(fiber.Map{
				"error": "failed to fetch process info",
			})
		}

		memInfo := &runtime.MemStats{}

		runtime.ReadMemStats(memInfo)

		return c.JSON(fiber.Map{
			"_": fiber.Map{
				"time":  "s",
				"space": "MiB",
			},
			"data": fiber.Map{
				"uptime":          info.Uptime,
				"procs":           info.Procs,
				"alloc_ram":       memInfo.Alloc / (1024 * 1024),
				"total_alloc_ram": memInfo.TotalAlloc / (1024 * 1024),
				"gc_cycles":       memInfo.NumGC,
			},
		})
	}
}
