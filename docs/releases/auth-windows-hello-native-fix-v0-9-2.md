---
agent: "ChatGPT 5.5"
created: "2026-07-02T11:11:06+00:00"
version: "0.9.2"
---

# Corrected Windows Hello native patch

The previous patch had malformed hunk headers (`@@` without line ranges), which caused:

```text
error: patch with only garbage
```

This version includes valid unified diff hunk headers.

Apply:

```powershell
git apply windows-hello-native-fix-v2.patch
```
