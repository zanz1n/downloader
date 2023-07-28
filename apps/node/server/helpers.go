package server

import "mime"

func sanitizeFileName(name string, contentType string) string {
	foundDot := false
	for i := len(name); i != 0; i-- {
		if name[i] == '.' {
			foundDot = true
			break
		}
	}

	if foundDot {
		return name
	}

	exts, err := mime.ExtensionsByType(contentType)

	if err == nil && exts != nil {
		if len(exts) > 0 {
			ext := exts[0]

			if len(ext) > 0 {
				if ext[0] != '.' {
					ext = "." + ext
				}
			}

			name = name + ext
		}
	}
	return name
}
