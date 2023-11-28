catalog-manifest
---

```
  Walks a warpforge catalog and joins information.

Usage: catalog-manifest --catalog-path <DIRECTORY> <COMMAND>

Commands:
  releases  Print a JSON object of references and ware IDs
  mirrors   Print a unified mirrors JSON object
  wares     Prints a list of ware IDs and fully qualified mirror locations
  help      Print this message or the help of the given subcommand(s)

Options:
  -c, --catalog-path <DIRECTORY>  The directory to walk. This is expected to be a warpforge catalog directory
  -h, --help                      Print help
````

## Build
Requires the rust ecosystem
```
  cargo build
```

## Example
```
  catalog-manifest -c ../warpsys-catalog/ mirrors
```
```
{
  "catalogmirrors.v1": {
  "byWare": {
    "tar:5TbZAHFB4BgXRAXcGkEedKDtD4sQNbjpRjgGw6kykgY1TLSvAjWpKPVwPAyWz4Tf8M": [
      "https://ftp.gnu.org/gnu/diffutils/diffutils-3.8.tar.xz"
    ],
    ...
  },
  "byModule": {
    "warpsys.org/diffutils": {
      "tar": [
        "ca+https://warpsys-wares.s3.fr-par.scw.cloud"
      ]
    },
  ...
  }
}
```