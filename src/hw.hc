# HitCode 语言全功能示例

function greet(str name)->return::str:
    say "你好,"
    say name
    end("greeted")

function add(int a, int b)->return::int:
    var int sum = a + b
    say "加法结果:"
    say sum
    end(sum)

function check_num(int n)->return::str:
    switch n:
        case 1:
            say "你输入的是1"
        case 2:
            say "你输入的是2"
        default:
            say "你输入的不是1也不是2"
    end("")

start:
    # 输入与字符串
    var str name = input("请输入你的名字：").to_str()
    call greet(name)

    # 变量、常量、列表
    var int x = 5
    var double y = 3.14
    var bool flag = true
    const int N = 3
    list int nums = [1, 2, 3, 4]
    list str words = ["hi", "hello", "bye"]

    # 数学运算
    x += 2
    y = y * 2
    say x
    say y

    # for 循环
    for n in nums:
        say n

    # if/while/do 控制
    if x > 5:
        say "x大于5"
    while x > 0:
        say x
        x -= 1
    do y > 10:
        say y
        y += 1

    # switch/case
    var int sel = 2
    switch sel:
        case 1:
            say "选择1"
        case 2:
            say "选择2"
        default:
            say "其他"

    # 函数调用与返回值
    var int result = call add(10, 20)
    say result
    call check_num(result)
end()