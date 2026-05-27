# Extract Instructions for Win 11

## Nodes
This does not include version since verion-info requires `wrestool`.

### Stable v1.1 as of 2026-05-25
1) Add oodle-data-shared.dll to this dir from FModel, e.g. `FModel\Output\.data\oodle-data-shared.dll` as of v2026-05-23
2) Run `dotnet script extract-win11-stable.cs`
3) Run `bun convert-no-version.js`


### EXP 
Build the tagged submodel version of CUEParse from FModel e.g.

```
git clone https://github.com/FabianFG/CUE4Parse external/CUE4Parse
cd external/CUE4Parse
git checkout cf74fc3
```

```
dotnet build external/CUE4Parse/CUE4Parse/CUE4Parse.csproj -c Release  -o external/CUE4Parse-publish -p:CopyLocalLockFileAssemblies=true
```

## Icons

### Stable v1.1 as of 2026-05-25
1) Run `dotnet script extract-win11-icon-stable.cs`
2) Browse icons in `/assets/icons/`
3) Dont commit these