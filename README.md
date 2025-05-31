# HitCode 脚本语言教程

HitCode 是一种基于缩进的脚本语言，支持变量/常量/列表声明、数学运算、字符串输出、自定义函数、流程控制（if/while/for/do/switch）等。所有块结构通过缩进自动识别，无需 end 关键字。

## 1. 基本语法
- 每个语句单独一行，块结构通过缩进（空格或Tab）表示。
- 注释以 `#` 或 `//` 开头。
- 所有字符串必须加双引号。

## 2. 变量与常量声明
```plaintext
var 类型 变量名 = 值
const 类型 常量名 = 值
```
示例：
```plaintext
var int x = 10
var double y = 3.14
var str s = "hello"
var bool b = true
const int N = 100
```

## 3. 列表声明
```plaintext
list 类型 名称 = [元素1, 元素2, ...]
```
示例：
```plaintext
list int nums = [1, 2, 3]
list str names = ["a", "b"]
```

## 4. 输出（say）
```plaintext
say 变量名
say "字符串"
```

## 5. 数学运算与赋值
```plaintext
x = x + 1
x += 2
y -= 1
```

## 6. 流程控制
### if
```plaintext
if 条件:
    ...
```
### while
```plaintext
while 条件:
    ...
```
### for
```plaintext
for 变量 in 列表名:
    ...
```
### do-while
```plaintext
do 条件:
    ...
```
### switch
```plaintext
switch 变量:
    case 值1:
        ...
    case 值2:
        ...
    default:
        ...
```

## 7. 函数定义与调用
```plaintext
function 函数名(类型 参数, ...)->return::类型:
    ...
    end(返回值)

# 调用
call 函数名(参数)
# 带返回值
var 类型 变量 = call 函数名(参数)
```

## 8. 输入
```plaintext
var str name = input("请输入你的名字：").to_str()
say name
```

## 9. 主程序块
```plaintext
start:
    ...
    call 函数名(参数)
    ...
end()
```

## 10. 完整示例
```plaintext
function greet(str who)->return::str:
    say "Hello,"
    say who
    end("ok")

start:
    var str name = input("请输入你的名字：").to_str()
    call greet(name)
end()
```

---
如需更多用法，请参考 hw.hc 示例或咨询开发者。
