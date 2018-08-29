//
//  LoadingView.swift
//  FourFours
//
//  Created by Kevin Guthrie on 8/16/18.
//  Copyright © 2018 Rook And Pawn Industries, Inc. All rights reserved.
//

import Foundation
import SpriteKit

class LoadingView : BaseView {
  let progressIndicator : ProgressBar
  
  override init() {
   
    self.progressIndicator = ProgressBar()
    super.init()
    addChild(progressIndicator)
  }
  
  required init?(coder aDecoder: NSCoder) {
    fatalError("init(coder:) has not been implemented")
  }
  
  override func layout(size: CGSize) {
    progressIndicator.setSize(size: CGSize(
      width: size.width / 5,
      height: size.width / 5 / 1.618))
    progressIndicator.position = CGPoint(x: size.width / 2.0, y: -size.height / 2.0)
  }
  
  func transitionToMainMenuView() {
    let mainMenu = MainMenuView()
    transitionTo(newView: mainMenu)
    mainMenu.setPresenter(presenter: getContext().bindToMainMenuView(view: mainMenu))
  }
  
  deinit {
    print("Dropping Loading view")
  }
}
