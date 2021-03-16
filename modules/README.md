# Fileserver
This service allows accessing files stored into the `public` folder through URLs. Npm version 6.14 or higher should be used.

## API
By default, an index of the files on the server is shown.
A file can be accessed by providing the path of the file relative to the `public` folder:
```
localhost:3000/path/to/file.txt
```

## Running the service
Before running the service install its dependencies by running `npm ci`.

The service can be started on the command line with `npm start`.
An automatic restart upon changes to source files can be enabled by starting the service with `npm run dev`.

By default the service runs in port `3000`, but you can provide another port through the `PORT` environment variable.
For example `PORT=80 npm start`. The metadata files assume the default port.