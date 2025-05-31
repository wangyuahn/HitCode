function test(str a)->return::str:
    say "Hello World!"
    say a
end("Hello World!")

start:
    call test("Hello World!")

    var str a = ""
    var str a = call test ("Hello Worldaaa!")
    say a

    var str b = ""
    var str b = input("Hello,enter your name:").to_str()
    switch b:
        case "":
            say "Hello World!"
        case "a":
            say "Hello Worldaaa!"
        case "b":
            say "Hello Worldbbb!"
        default:
            say "Hello World!"
    
end()