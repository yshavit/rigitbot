# Ignoring `backend.tf`

The file `backend.tf` is being ignored. To manage state using Hashicorp Cloud, add a local `backend.tf` that looks something like:

```
terraform {
  backend "remote" {
    organization = "<your-organization-name>"
    workspaces {
      name = "your-workspace-name"
    }
  }
}
```
