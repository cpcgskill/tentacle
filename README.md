# tentacle

一个用于项目构建的编程语言

现在啥都没完善不建议用于正式项目

## 目录

- [快速开始](#快速开始)
- [版权说明](#版权说明)

### 快速开始

1. 下载可执行文件并将其放于可以搜索到的路径下
2. 在当前目录下创建main.tentacle文件
3. 向文件中写入以下代码后执行"tentacle main"

```commandline
target $clean:
    message target main is $clean
target $main:$clean
    message target main is $main
```

### 版权说明

该项目签署了Apache-2.0 授权许可，详情请参阅 LICENSE