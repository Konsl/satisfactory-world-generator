idk if i can include the default world resource node data

extracting it using these scripts should be relatively easy tho

## Notes
- At the time of writing, the latest version of CUE4Parse available on nuget.org is `1.2.2`, which doesn't parse UE 5.6 game data correctly.
  Either compile it yourself or use version `1.2.2.21` from the [GitHub NuGet registry](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-nuget-registry#installing-a-package).
- You need a version of the oodle shared library (e.g. `oodle-data-shared.dll`).
  Take the one from the UE GitHub repo (if you have access to that) or run FModel once
  and take the file from `FModel\Output\.data\oodle-data-shared.dll`.
- You can try running the script with `dotnet script extract.cs` if you have that installed.
  It might default to using `net8.0`, which is too old for CUE4Parse.
  In that case, just create a C# project, paste the contents of `extract.cs`
  and install the NuGet packages. (remove the `#r` lines)
- Once you have successfully ran the extraction script, copy the `textures` directory to `src/`
  and run `convert.js` using bun with the `extracted-resources.json` file placed in `scripts/`.

