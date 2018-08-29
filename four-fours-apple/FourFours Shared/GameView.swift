//
//  GameView.swift
//  FourFours
//
//  Created by Kevin Guthrie on 8/21/18.
//  Copyright © 2018 Rook And Pawn Industries, Inc. All rights reserved.
//

import Foundation
import SpriteKit

class GameView : BaseView {
  
  var layoutHandlers: [LayoutHandler] = []
  
  static var z = 1;
  
  override init() {
    super.init()
  }
  
  required init?(coder aDecoder: NSCoder) {
    fatalError("init(coder:) has not been implemented")
  }
  
  func addLayoutHandler(_ handler: LayoutHandler) -> HandlerRegistration {
    DispatchQueue.main.sync {
      self.layoutHandlers.append(handler)
    }
    
    return HandlerRegistration(deregister_callback: {
      self.removeHandler(handler)
    })
      
  }
  
  func removeHandler(_ handler: LayoutHandler) {
    DispatchQueue.main.sync {
      if let index = self.layoutHandlers.index(of: handler) {
        self.layoutHandlers.remove(at: index)
      }
    }
  }
  
  override func layout(size: CGSize) {
    layoutHandlers.forEach { (handler) in
      handler.onLayout(width: Int64(size.width), height: Int64(size.height))
    }
  }
  
  func createSprite() -> Sprite {
    let result = Sprite()
    DispatchQueue.main.sync {
      result.zPosition = CGFloat(GameView.z)
      GameView.z += 1
      addChild(result)
    }
    return result
  }
  
  deinit {
    print("Dropping GameView")
  }
}
