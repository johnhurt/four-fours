//
//  MainMenu.swift
//  Enchantron
//
//  Created by Kevin Guthrie on 7/19/18.
//  Copyright © 2018 Rook And Pawn Industries, Inc. All rights reserved.
//

import Foundation
import SpriteKit

class MainMenuView : BaseView {
  
  let startNewGameButton : Button
  
  
  override init() {
    
    let startNewGameButton = Button()
    startNewGameButton.setFillColor(fillColor: SKColor.cyan)
    
    self.startNewGameButton = startNewGameButton
    
    super.init()
    
    addChild(startNewGameButton)
  }
  
  required init?(coder aDecoder: NSCoder) {
    fatalError("init(coder:) has not been implemented")
  }
  
  func transitionToGameView() {
    let gameView = GameView()
    gameView.setPresenter(presenter: getContext().bindToGameView(view: gameView))
    transitionTo(newView: gameView)
  }
  
  override func layout(size: CGSize) {
    startNewGameButton.setSize(size: CGSize(
        width: size.width / 5,
        height: size.width / 5 / 1.618))
    startNewGameButton.position = CGPoint(x: size.width / 2.0, y: -size.height / 2.0)
  }
  
  deinit {
    print("Dropping MainMenuView")
  }
}

