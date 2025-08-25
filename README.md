# quote-bot
This is a Discord bot written in Rust that takes quotes given by the user to then store in a SQL database. From there, a user may ask the bot to retrieve random quotes from the database or retrieve random quotes from a specified user.

``!quote`` - Outputs a random quote from the database.
``!quote <name>`` - Outputs a random quote from the given name.
``!addquote "Quote text" - <name>`` - Adds a given quote text from a given name to the database. Gives a "thumbs-up" emoticon when the quote is accepted.
``!help`` - Outputs all of the functions for using the quote-bot.
``!help <function>`` - Explains what the given function does (!help !quote).
