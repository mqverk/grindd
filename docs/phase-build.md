# Build System Notes

- Build files support `FROM`, `RUN`, `COPY`, and `CMD` instructions.
- Cache keys derive from instruction text and content digest for `COPY` source files.
- Build cache metadata is persisted under the daemon state directory.
