# crossgate-example

# 先下载crossgate

git clone https://github.com/ddx2x/crossgate.git

git clone 本项目

## build

```
docker buildx build \
  --platform linux/arm/v7,linux/arm64/v8,linux/386,linux/amd64,linux/ppc64le \
  -t stream:0.1\
  -f Dockerfile.stream \
  .
```
