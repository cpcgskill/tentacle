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

```python
$target_index = 15
message test_command1
message test_command2 target_index ("test expr" + " 测试整数格式化 " + -15 + " 测试浮点数格式化 " + 10.5)
message test_command3 target_index ($target_index+-1)
target $clean:
    message target clean is $clean
target $build: $clean
    "用于测试的target"
    message target build is $build
message test_print_target $build

$select = 0
if $select == 0:
    message test eq and not eq "$select is" 0
elif $select == 1:
    message test eq and not eq "$select is" 1
elif $select == 2:
    message test eq and not eq "$select is" 2
else:
    message test eq and not eq "$select is not (0, 1, 2)"

$select = "a"
if $select == "a":
    message test eq and not eq "$select is" a
elif $select == "b":
    message test eq and not eq "$select is" b
elif $select == "c":
    message test eq and not eq "$select is" c
else:
    message test eq and not eq "$select is not (a, b, c)"


message test_list [1, "1", 1+2*3, (1+2)]

for $i in [0, 1, 2, 3, 4, 5, 6]:
    message for index $i
```

### 版权说明

该项目签署了Apache-2.0 授权许可，详情请参阅 LICENSE