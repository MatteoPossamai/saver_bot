# Idea

## Note
- Can call the Recycle tool to convert into coin all my backpack available. 
- To produce coin can use tree, rock -> Garbage -> Coin

- Use the chart tool that allows to save all the banks
- use the asphaltinator to create street between banks
- Use charting tools to save all banks 
- Use asphaltinator to connect the banks 
- While moving, also get all the useful surroundings, with destroy tool
- Use best path to reach banks
- Use basic search to find resources when you do not have any

## State transition
- Start: CoinCollecting

- CoinCollecting: 
    - Find A bank: 
        - Add to the unconnected banks. Save it. If there is at least  another one saved or in unconnected, go in RockCollecting
    - Fill the backpack: 
        - Use the tool to exchange everything to Coin, if there are less then 10 Coin and go on, otherwise, go to Saving
- RockCollecting: 
    - Fill the backpack: start to create a new project or keeps on doing one that previously was to do. Once done, returns in RockCollecting if there are still to connect or the returned thing is not done, otherwise switch to coin collecting. 
- Connecting: 
    - Creating the thing with the tool, then back to RockCollecting if there are still to connect, otherwise back to CoinCollecting
- Saving:
    - Putting the coin into a bank. If I do not have any bank, I wander randomly until I find one. Once saved at least 70% of the current money, go back to CoinCollecting, or Connecting if found enough things. 
- Enjoying:
    - Once goal reached the robot simply celebrate. 
- Trading: 
    - Once done, if it has more than 12 coins, he searches for a place to store them, so goes into saving, else again CoinCollecting