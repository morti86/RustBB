# locales/en-US/main.ftl
welcome = Welcome, {$userName}!
greeting = Hello, world!
login-button = Log In
logout-button = Log Out
items-count = You have { $count ->
    [one] one item
   *[other] {$count} items
}

time-elapsed = Time elapsed: { $hours }h { $minutes }m { $seconds }s
current-date = Today is { DATETIME($date, month: "long", year: "numeric", day: "numeric") }
