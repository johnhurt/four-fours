//
//  LoadingView.swift
//  FourFours
//
//  Created by Kevin Guthrie on 8/16/18.
//  Copyright © 2018 Rook And Pawn Industries, Inc. All rights reserved.
//

import Foundation
import SpriteKit

class LoadingView : BaseView<WrappedLoadingPresenter> {
  let progressIndicator : ProgressBar
  
  override init() {
   
    self.progressIndicator = ProgressBar(size: CGSize(width: 400, height: 200))
    super.init()
    addChild(progressIndicator)
  }
  
  required init?(coder aDecoder: NSCoder) {
    fatalError("init(coder:) has not been implemented")
  }
  
  func transitionToMainMenuView() {
    let mainMenu = MainMenuView()
    transitionTo(newView: mainMenu)
    mainMenu.setPresenter(presenter: getContext().bindToMainMenuView(view: mainMenu))
  }
}
