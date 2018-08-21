//
//  BaseView.swift
//  FourFours
//
//  Created by Kevin Guthrie on 8/17/18.
//  Copyright © 2018 Rook And Pawn Industries, Inc. All rights reserved.
//

import Foundation
import SpriteKit

class BaseView<P>: SKNode {
  
  private var presenter : P?
  private var ctx : ApplicationContext?
  private var transitionService : TransitionService?
  
  override init() {
    super.init()
  }
  
  required init?(coder aDecoder: NSCoder) {
    fatalError("init(coder:) has not been implemented")
  }
  
  func getContext() -> ApplicationContext {
    return ctx!
  }
  
  func transitionTo<T>(newView: BaseView<T>) {
    newView.initializeCtx(ctx: self.ctx!, transitionService: self.transitionService!)
    transitionService?.transition(view: newView)
    self.presenter = nil
  }
  
  func initializeCtx(ctx : ApplicationContext,
                  transitionService : TransitionService) {
    self.ctx = ctx
    self.transitionService = transitionService
  }
  
  func setPresenter(presenter: P) {
    self.presenter = presenter
  }
  
  func unsetPresenter() {
    self.presenter = nil
  }
}
