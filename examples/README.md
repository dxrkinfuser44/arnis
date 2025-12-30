# Arnis Examples

This directory contains example scripts and usage patterns for Arnis.

## Pre-Caching Examples

### batch_precache.sh (Linux/macOS)

A bash script that demonstrates how to pre-cache multiple regions for offline generation.

**Usage:**

```bash
chmod +x batch_precache.sh
./batch_precache.sh
```

**Customization:**

Edit the `REGIONS` array in the script to add your own regions:

```bash
declare -a REGIONS=(
    "Region Name|min_lat,min_lng,max_lat,max_lng"
    "Another Region|40.7,-74.0,40.8,-73.9"
)
```

### batch_precache.bat (Windows)

A Windows batch script with the same functionality as the bash version.

**Usage:**

Double-click `batch_precache.bat` or run from command prompt:

```cmd
batch_precache.bat
```

**Customization:**

Edit the `REGION[n]` variables in the script:

```batch
set "REGION[0]=Region Name^|min_lat,min_lng,max_lat,max_lng"
set "REGION[1]=Another Region^|40.7,-74.0,40.8,-73.9"
```

## Features Demonstrated

### Pre-Caching Workflow

1. **Download**: Fetch OSM and elevation data from APIs
2. **Cache**: Store data locally for offline use
3. **Generate**: Create Minecraft worlds from cached data

### Use Cases

- **Large Area Generation**: Cache data first to avoid re-downloading if generation fails
- **Batch Processing**: Pre-cache multiple regions, then generate worlds when convenient
- **Offline Work**: Download data when you have internet, generate worlds later
- **Multiple Variations**: Cache once, generate different versions (with/without interiors, etc.)

## Additional Examples

### Simple Pre-Cache

Cache a single region:

```bash
# Linux/macOS
arnis --cache-only --bbox="40.7,-74.0,40.8,-73.9" --scale=1.0 --terrain

# Windows
arnis.exe --cache-only --bbox="40.7,-74.0,40.8,-73.9" --scale=1.0 --terrain
```

### Generate from Cache

List available caches:

```bash
arnis --list-caches
```

Generate world from a specific cache:

```bash
# Linux/macOS
arnis --from-cache cache_40700_-74000_40800_-73900_1234567890 \
      --path="$HOME/.minecraft/saves/MyWorld" \
      --interior --roof

# Windows
arnis.exe --from-cache cache_40700_-74000_40800_-73900_1234567890 ^
          --path="%APPDATA%\.minecraft\saves\MyWorld" ^
          --interior --roof
```

### Cache Management

Delete a specific cache:

```bash
arnis --delete-cache cache_40700_-74000_40800_-73900_1234567890
```

Clear all caches:

```bash
arnis --clear-caches
```

## Tips

1. **API Rate Limiting**: Add delays between requests when batch caching (see scripts for examples)
2. **Disk Space**: Monitor cache size with `--list-caches` to see total storage used
3. **Cache Location**: Use `--cache-dir` to specify a custom cache directory if needed
4. **Naming**: Cache entries are automatically named using location data when available

## Contributing

Have a useful Arnis workflow or script? Contributions are welcome!

1. Fork the repository
2. Add your example to this directory
3. Update this README
4. Submit a pull request

## Support

For help with these examples or Arnis in general:

- Read the main documentation: [../PRE_CACHING.md](../PRE_CACHING.md)
- Check the GitHub issues: https://github.com/louis-e/arnis/issues
- Join the Discord: https://discord.gg/mA2g69Fhxq

## License

These examples are part of Arnis and licensed under Apache-2.0.