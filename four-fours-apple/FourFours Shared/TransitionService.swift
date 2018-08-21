//
//  TransitionService.swift
//  Enchantron
//
//  Created by Kevin Guthrie on 7/25/18.
//  Copyright © 2018 Rook And Pawn Industries, Inc. All rights reserved.
//

import Foundation
import SpriteKit

class TransitionService {
  
  let transitionClosure: (SKNode) -> Void
  
  init(transitionClosure: @escaping (SKNode) -> Void ) {
    self.transitionClosure = transitionClosure
  }
  
  func transition(view: SKNode) {
    DispatchQueue.main.async {
      self.transitionClosure(view)
    }
  }
  
  deinit{
    print("Dropping Transition Service")
  }
}
